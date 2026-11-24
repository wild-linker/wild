/// Evaluation of linker script ASSERT commands after layout is complete.
///
/// NOTE: ASSERT expression evaluation currently supports a subset of GNU ld expression
/// features. Symbol resolution and full location counter semantics (e.g. ALIGN with a non-zero
/// current address) will be implemented in future work.
use crate::bail;
use crate::error::Context;
use crate::error::Result;
use crate::grouping::Group;
use crate::input_data::InputRef;
use crate::layout;
use crate::layout::FileLayoutState;
use crate::layout::GroupState;
use crate::layout::HandlerData;
use crate::layout::InputSectionPositions;
use crate::layout::MemoryRegion;
use crate::layout::OutputRecordLayout;
use crate::linker_script::Expression;
use crate::output_section_id::OutputSectionId;
use crate::output_section_id::OutputSections;
use crate::output_section_id::SectionName;
use crate::output_section_map::OutputSectionMap;
use crate::output_section_part_map::OutputSectionPartMap;
use crate::parsing::SymbolLoc;
use crate::parsing::SymbolPlacement;
use crate::part_id::PartId;
use crate::platform::Args;
use crate::platform::Platform;
use crate::platform::SectionAttributes;
use crate::symbol::UnversionedSymbolName;
use crate::symbol_db::SymbolDb;
use crate::symbol_db::SymbolId;
use hashbrown::HashMap;
use std::cell::OnceCell;

/// Compute 1-based line number by counting newlines before `remainder` in `file_bytes`.
fn line_number(file_bytes: &[u8], remainder: &[u8]) -> u32 {
    let parsed_len = file_bytes.len().saturating_sub(remainder.len());
    let consumed = &file_bytes[..parsed_len];
    consumed.iter().filter(|&&b| b == b'\n').count() as u32 + 1
}

#[derive(Clone, Default)]
pub(crate) struct ResolvedLocationCounter {
    pub(crate) value: u64,
    pub(crate) section_offset: Option<u64>,
}

pub(crate) enum SymbolValue {
    Absolute(u64),
    PartRelative {
        part_id: PartId,
        offset: u64,
    },
    SectionRelative {
        section_id: OutputSectionId,
        address: u64,
    },
}

impl SymbolValue {
    pub(crate) fn new(address: u64, section_id: Option<OutputSectionId>) -> Self {
        match section_id {
            Some(section_id) => SymbolValue::SectionRelative {
                section_id,
                address,
            },
            None => SymbolValue::Absolute(address),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExpressionValueKind {
    Number,
    Address,
    Absolute,
    Relative(OutputSectionId),
}

impl ExpressionValueKind {
    pub(crate) fn is_absolute(self) -> bool {
        matches!(self, ExpressionValueKind::Absolute)
    }

    pub(crate) fn is_number(self) -> bool {
        matches!(self, ExpressionValueKind::Number)
    }

    pub(crate) fn section_id(self) -> Option<OutputSectionId> {
        match self {
            ExpressionValueKind::Relative(sec) => Some(sec),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EvaluatedValue {
    pub(crate) value: u64,
    pub(crate) value_kind: ExpressionValueKind,
}

impl EvaluatedValue {
    fn number(value: u64) -> Self {
        Self {
            value,
            value_kind: ExpressionValueKind::Number,
        }
    }

    pub(crate) fn is_absolute(&self) -> bool {
        self.value_kind.is_absolute()
    }

    pub(crate) fn section_id(&self) -> Option<OutputSectionId> {
        self.value_kind.section_id()
    }
}

fn evaluate_section_address(
    target_section: OutputSectionId,
    address: u64,
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    name: &[u8],
) -> Result<EvaluatedValue> {
    let section_base = section_layouts.get(target_section).mem_offset;
    let offset = address.checked_sub(section_base).with_context(|| {
        format!(
            "address of '{}' is before its output section",
            String::from_utf8_lossy(name)
        )
    })?;
    Ok(EvaluatedValue {
        value: offset,
        value_kind: ExpressionValueKind::Relative(target_section),
    })
}

fn evaluate_symbol_value<P: Platform>(
    symbol_value: &SymbolValue,
    loc: &SymbolLoc,
    output_sections: &OutputSections<'_, P>,
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    laid_out_mem_offsets: &OutputSectionPartMap<Option<u64>>,
    name: &[u8],
) -> Result<EvaluatedValue> {
    let current_section = loc
        .relative_section_id()
        .map(|id| output_sections.primary_output_section(id));
    match symbol_value {
        SymbolValue::Absolute(value) => Ok(EvaluatedValue {
            value: *value,
            value_kind: if current_section.is_some() {
                ExpressionValueKind::Number
            } else {
                ExpressionValueKind::Absolute
            },
        }),
        SymbolValue::SectionRelative {
            section_id,
            address,
        } => {
            let primary_section = output_sections.primary_output_section(*section_id);
            let section_base = section_layouts.get(primary_section).mem_offset;
            let offset = address.wrapping_sub(section_base);
            Ok(EvaluatedValue {
                value: offset,
                value_kind: ExpressionValueKind::Relative(primary_section),
            })
        }
        SymbolValue::PartRelative { part_id, offset } => {
            let Some(part_address) = laid_out_mem_offsets.get(*part_id) else {
                bail!(
                    "cannot resolve address of symbol '{}' because its output section part has not been laid out yet",
                    String::from_utf8_lossy(name)
                );
            };
            let address = part_address + offset;
            let symbol_section =
                output_sections.primary_output_section(part_id.output_section_id::<P>());
            evaluate_section_address(symbol_section, address, section_layouts, name)
        }
    }
}

fn arithmetic_binary_op(
    mut val_l: u64,
    mut kind_l: ExpressionValueKind,
    mut val_r: u64,
    mut kind_r: ExpressionValueKind,
    current_section: Option<OutputSectionId>,
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    op: impl FnOnce(u64, u64) -> Result<u64>,
) -> Result<EvaluatedValue> {
    let is_in_section = current_section.is_some();
    let normalize_rel = |val: &mut u64, kind: &mut ExpressionValueKind, sec: OutputSectionId| {
        *val = val.wrapping_add(section_layouts.get(sec).mem_offset);
        *kind = ExpressionValueKind::Address;
    };

    match (kind_l, kind_r) {
        (ExpressionValueKind::Relative(sec_l), ExpressionValueKind::Relative(sec_r)) => {
            if sec_l != sec_r || !is_in_section {
                normalize_rel(&mut val_l, &mut kind_l, sec_l);
                normalize_rel(&mut val_r, &mut kind_r, sec_r);
            }
        }
        (
            ExpressionValueKind::Relative(sec_l),
            ExpressionValueKind::Absolute | ExpressionValueKind::Address,
        ) => {
            normalize_rel(&mut val_l, &mut kind_l, sec_l);
        }
        (
            ExpressionValueKind::Absolute | ExpressionValueKind::Address,
            ExpressionValueKind::Relative(sec_r),
        ) => {
            normalize_rel(&mut val_r, &mut kind_r, sec_r);
        }
        (ExpressionValueKind::Relative(sec_l), ExpressionValueKind::Number) if !is_in_section => {
            normalize_rel(&mut val_l, &mut kind_l, sec_l);
        }
        (ExpressionValueKind::Number, ExpressionValueKind::Relative(sec_r)) if !is_in_section => {
            normalize_rel(&mut val_r, &mut kind_r, sec_r);
        }
        _ => {}
    }

    let value_kind = if is_in_section {
        match (kind_l, kind_r) {
            (ExpressionValueKind::Number, ExpressionValueKind::Number) => {
                ExpressionValueKind::Number
            }
            (ExpressionValueKind::Relative(_), ExpressionValueKind::Relative(_)) => {
                ExpressionValueKind::Number
            }
            (
                ExpressionValueKind::Absolute | ExpressionValueKind::Address,
                ExpressionValueKind::Absolute | ExpressionValueKind::Address,
            ) => ExpressionValueKind::Number,
            (ExpressionValueKind::Relative(sec), _) | (_, ExpressionValueKind::Relative(sec)) => {
                ExpressionValueKind::Relative(sec)
            }
            (ExpressionValueKind::Absolute, _) | (_, ExpressionValueKind::Absolute) => {
                ExpressionValueKind::Absolute
            }
            _ => ExpressionValueKind::Address,
        }
    } else if kind_l.is_number() && kind_r.is_number() {
        ExpressionValueKind::Number
    } else if matches!(kind_l, ExpressionValueKind::Address)
        && matches!(kind_r, ExpressionValueKind::Number)
        || matches!(kind_l, ExpressionValueKind::Number)
            && matches!(kind_r, ExpressionValueKind::Address)
    {
        ExpressionValueKind::Address
    } else {
        ExpressionValueKind::Absolute
    };

    Ok(EvaluatedValue {
        value: op(val_l, val_r)?,
        value_kind,
    })
}

fn compare_binary_op(
    mut val_l: u64,
    kind_l: ExpressionValueKind,
    mut val_r: u64,
    kind_r: ExpressionValueKind,
    current_section: Option<OutputSectionId>,
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    op: impl FnOnce(u64, u64) -> bool,
) -> EvaluatedValue {
    let to_address = |val: &mut u64, kind: ExpressionValueKind| {
        if let ExpressionValueKind::Relative(sec) = kind {
            *val = val.wrapping_add(section_layouts.get(sec).mem_offset);
        }
    };

    if current_section.is_some() {
        match (kind_l, kind_r) {
            (ExpressionValueKind::Relative(sec_l), ExpressionValueKind::Relative(sec_r)) => {
                if sec_l != sec_r {
                    to_address(&mut val_l, kind_l);
                    to_address(&mut val_r, kind_r);
                }
            }
            (
                ExpressionValueKind::Relative(_),
                ExpressionValueKind::Absolute | ExpressionValueKind::Address,
            ) => {
                to_address(&mut val_l, kind_l);
            }
            (
                ExpressionValueKind::Absolute | ExpressionValueKind::Address,
                ExpressionValueKind::Relative(_),
            ) => {
                to_address(&mut val_r, kind_r);
            }
            _ => {}
        }
    } else {
        to_address(&mut val_l, kind_l);
        to_address(&mut val_r, kind_r);
    }

    let res = op(val_l, val_r);

    let value_kind = if current_section.is_some() {
        ExpressionValueKind::Number
    } else {
        ExpressionValueKind::Absolute
    };

    EvaluatedValue {
        value: u64::from(res),
        value_kind,
    }
}

fn min_max_binary_op(
    mut val_l: u64,
    l_kind: ExpressionValueKind,
    mut val_r: u64,
    r_kind: ExpressionValueKind,
    current_section: Option<OutputSectionId>,
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    op: impl FnOnce(u64, u64) -> u64,
) -> EvaluatedValue {
    if let Some(curr_sec) = current_section
        && !l_kind.is_absolute()
        && !r_kind.is_absolute()
        && (matches!(l_kind, ExpressionValueKind::Address)
            || matches!(r_kind, ExpressionValueKind::Address))
    {
        let curr_base = section_layouts.get(curr_sec).mem_offset;
        let to_addr = |val: u64, kind: ExpressionValueKind| match kind {
            ExpressionValueKind::Relative(sec) => {
                section_layouts.get(sec).mem_offset.wrapping_add(val)
            }
            ExpressionValueKind::Number => curr_base.wrapping_add(val),
            _ => val,
        };
        let addr_l = to_addr(val_l, l_kind);
        let addr_r = to_addr(val_r, r_kind);
        let res_addr = op(addr_l, addr_r);
        return EvaluatedValue {
            value: res_addr.wrapping_sub(curr_base),
            value_kind: ExpressionValueKind::Relative(curr_sec),
        };
    }

    let is_in_section = current_section.is_some();
    let value_kind = if l_kind.section_id().is_some() && l_kind.section_id() == r_kind.section_id()
    {
        l_kind
    } else if l_kind.is_number() && r_kind.is_number() {
        ExpressionValueKind::Number
    } else if is_in_section && l_kind.section_id().is_some() && r_kind.is_number() {
        l_kind
    } else if is_in_section && l_kind.is_number() && r_kind.section_id().is_some() {
        r_kind
    } else {
        let to_address = |val: &mut u64, kind: ExpressionValueKind| {
            if let ExpressionValueKind::Relative(sec) = kind {
                *val = val.wrapping_add(section_layouts.get(sec).mem_offset);
            }
        };
        to_address(&mut val_l, l_kind);
        to_address(&mut val_r, r_kind);
        ExpressionValueKind::Absolute
    };

    EvaluatedValue {
        value: op(val_l, val_r),
        value_kind,
    }
}

fn evaluate_location<'data, P: Platform>(
    expr_loc: &SymbolLoc,
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    output_sections: &OutputSections<'data, P>,
    resolved_location_counters: &[ResolvedLocationCounter],
) -> Result<EvaluatedValue> {
    let value_kind = expr_loc
        .section_id()
        .map_or(ExpressionValueKind::Number, |id| {
            ExpressionValueKind::Relative(output_sections.primary_output_section(id))
        });
    let value = match expr_loc {
        SymbolLoc::SectionStartRelative(_) => 0,
        SymbolLoc::SectionEndRelative(id) | SymbolLoc::SectionEnd(id) => {
            let primary_id = output_sections.primary_output_section(*id);
            let primary_start = section_layouts.get(primary_id).mem_offset;
            let id_layout = section_layouts.get(*id);
            let id_end = id_layout.mem_offset + id_layout.mem_size;
            id_end - primary_start
        }
        SymbolLoc::FirstSection | SymbolLoc::None => 0,
        SymbolLoc::LocationCounter(idx, section_id) => {
            let entry = resolved_location_counters.get(*idx).with_context(|| {
                format!(
                    "location counter index {idx} out of range (len: {})",
                    resolved_location_counters.len()
                )
            })?;
            if section_id.is_none() {
                entry.value
            } else {
                entry.section_offset.unwrap_or(0)
            }
        }
    };
    Ok(EvaluatedValue { value, value_kind })
}

pub(crate) struct ExpressionContext<'scope, 'data, P: Platform> {
    pub(crate) expr_loc: &'scope SymbolLoc,
    pub(crate) input_ref: Option<&'scope InputRef<'data>>,
    pub(crate) section_layouts: &'scope OutputSectionMap<OutputRecordLayout>,
    pub(crate) output_sections: &'scope OutputSections<'data, P>,
    pub(crate) memory_regions: &'scope HashMap<&'scope [u8], MemoryRegion>,
    pub(crate) symbol_db: &'scope SymbolDb<'data, P>,
    pub(crate) sizeof_headers: u64,
    pub(crate) resolved_location_counters: &'scope [ResolvedLocationCounter],
    pub(crate) laid_out_mem_offsets: &'scope OutputSectionPartMap<Option<u64>>,
}

impl<'data, P: Platform> ExpressionContext<'_, 'data, P> {
    pub(crate) fn evaluate(
        &self,
        expr: &Expression<'data>,
        symbol_resolution_callback: &mut dyn FnMut(&[u8]) -> Result<SymbolValue>,
    ) -> Result<EvaluatedValue> {
        let mut evaluated = self.evaluate_value(expr, symbol_resolution_callback)?;
        let offset = if let ExpressionValueKind::Relative(sec) = evaluated.value_kind {
            self.section_layouts.get(sec).mem_offset
        } else if let ExpressionValueKind::Number = evaluated.value_kind
            && let Some(primary_id) = self.current_section()
        {
            self.section_layouts.get(primary_id).mem_offset
        } else {
            0
        };

        if evaluated.value_kind == ExpressionValueKind::Number {
            evaluated.value_kind = self
                .current_section()
                .map_or(ExpressionValueKind::Absolute, ExpressionValueKind::Relative);
        }
        evaluated.value = evaluated.value.wrapping_add(offset);
        Ok(evaluated)
    }

    fn current_section(&self) -> Option<OutputSectionId> {
        self.expr_loc
            .relative_section_id()
            .map(|id| self.output_sections.primary_output_section(id))
    }

    fn evaluate_value(
        &self,
        expr: &Expression<'data>,
        symbol_resolution_callback: &mut dyn FnMut(&[u8]) -> Result<SymbolValue>,
    ) -> Result<EvaluatedValue> {
        macro_rules! eval_binary_op {
            ($l:expr, $r:expr, $func:ident, $op:expr) => {{
                let left = self.evaluate_value($l, symbol_resolution_callback)?;
                let right = self.evaluate_value($r, symbol_resolution_callback)?;
                $func(
                    left.value,
                    left.value_kind,
                    right.value,
                    right.value_kind,
                    self.current_section(),
                    self.section_layouts,
                    $op,
                )
            }};
        }

        match expr {
            Expression::Number(n) => Ok(EvaluatedValue::number(*n)),
            Expression::LocationCounter => evaluate_location(
                self.expr_loc,
                self.section_layouts,
                self.output_sections,
                self.resolved_location_counters,
            ),
            Expression::Symbol(name) => {
                let value = symbol_resolution_callback(name)?;
                evaluate_symbol_value(
                    &value,
                    self.expr_loc,
                    self.output_sections,
                    self.section_layouts,
                    self.laid_out_mem_offsets,
                    name,
                )
            }
            Expression::Add(l, r) => {
                eval_binary_op!(l, r, arithmetic_binary_op, |a, b| Ok(a.wrapping_add(b)))
            }
            Expression::Subtract(l, r) => {
                eval_binary_op!(l, r, arithmetic_binary_op, |a, b| Ok(a.wrapping_sub(b)))
            }
            Expression::Multiply(l, r) => {
                eval_binary_op!(l, r, arithmetic_binary_op, |a, b| Ok(a.wrapping_mul(b)))
            }
            Expression::Divide(l, r) => eval_binary_op!(l, r, arithmetic_binary_op, |a, b| {
                if b == 0 {
                    bail!("Division by zero in linker script expression");
                }
                Ok(((a as i64).wrapping_div(b as i64)) as u64)
            }),
            Expression::Modulo(l, r) => eval_binary_op!(l, r, arithmetic_binary_op, |a, b| {
                if b == 0 {
                    bail!("Modulo by zero in linker script expression");
                }
                Ok(((a as i64).wrapping_rem(b as i64)) as u64)
            }),

            // Comparisons return 1 (true) or 0 (false)
            Expression::LessThan(l, r) => {
                Ok(eval_binary_op!(l, r, compare_binary_op, |a, b| a < b))
            }
            Expression::GreaterThan(l, r) => {
                Ok(eval_binary_op!(l, r, compare_binary_op, |a, b| a > b))
            }
            Expression::LessEqual(l, r) => {
                Ok(eval_binary_op!(l, r, compare_binary_op, |a, b| a <= b))
            }
            Expression::GreaterEqual(l, r) => {
                Ok(eval_binary_op!(l, r, compare_binary_op, |a, b| a >= b))
            }
            Expression::Equal(l, r) => Ok(eval_binary_op!(l, r, compare_binary_op, |a, b| a == b)),
            Expression::NotEqual(l, r) => {
                Ok(eval_binary_op!(l, r, compare_binary_op, |a, b| a != b))
            }

            Expression::Sizeof(name) => Ok(EvaluatedValue::number(section_size(
                name,
                self.section_layouts,
                self.output_sections,
            ))),
            Expression::Alignof(name) => Ok(EvaluatedValue::number(section_align(
                name,
                self.section_layouts,
                self.output_sections,
            ))),
            Expression::Addr(name) => {
                section_address(name, self.section_layouts, self.output_sections)
            }
            Expression::Loadaddr(name) => Ok(EvaluatedValue {
                value: section_load_address(name, self.section_layouts, self.output_sections)?,
                value_kind: ExpressionValueKind::Absolute,
            }),
            Expression::Align(exponent, expr) => {
                let align = self
                    .evaluate_value(exponent, symbol_resolution_callback)?
                    .value;
                if align == 0 {
                    bail!("ALIGN(0) is invalid");
                }
                let is_dot = expr.is_none();
                let mut evaluated = match expr.as_ref() {
                    Some(e) => {
                        let mut evaluated = self.evaluate_value(e, symbol_resolution_callback)?;
                        if self.current_section().is_none() {
                            if let Some(sec) = evaluated.section_id() {
                                evaluated.value = evaluated
                                    .value
                                    .wrapping_add(self.section_layouts.get(sec).mem_offset);
                            }
                            evaluated.value_kind = ExpressionValueKind::Absolute;
                        }
                        evaluated
                    }
                    None => evaluate_location(
                        self.expr_loc,
                        self.section_layouts,
                        self.output_sections,
                        self.resolved_location_counters,
                    )?,
                };
                evaluated.value = if is_dot && let Some(sec) = evaluated.section_id() {
                    let base = self.section_layouts.get(sec).mem_offset;
                    base.wrapping_add(evaluated.value)
                        .next_multiple_of(align)
                        .wrapping_sub(base)
                } else {
                    evaluated.value.next_multiple_of(align)
                };
                Ok(evaluated)
            }
            Expression::Min(l, r) => Ok(eval_binary_op!(l, r, min_max_binary_op, u64::min)),
            Expression::Max(l, r) => Ok(eval_binary_op!(l, r, min_max_binary_op, u64::max)),
            Expression::BitwiseAnd(l, r) => {
                eval_binary_op!(l, r, arithmetic_binary_op, |a, b| Ok(a & b))
            }
            Expression::BitwiseOr(l, r) => {
                eval_binary_op!(l, r, arithmetic_binary_op, |a, b| Ok(a | b))
            }
            Expression::BitwiseXor(l, r) => {
                eval_binary_op!(l, r, arithmetic_binary_op, |a, b| Ok(a ^ b))
            }
            Expression::LeftShift(l, r) => {
                eval_binary_op!(l, r, arithmetic_binary_op, |a, b| {
                    Ok(a.wrapping_shl(b as u32))
                })
            }
            Expression::RightShift(l, r) => {
                eval_binary_op!(l, r, arithmetic_binary_op, |a, b| {
                    Ok(a.wrapping_shr(b as u32))
                })
            }
            Expression::LogicalAnd(l, r) => {
                Ok(eval_binary_op!(l, r, compare_binary_op, |a, b| a != 0 && b != 0))
            }
            Expression::LogicalOr(l, r) => {
                Ok(eval_binary_op!(l, r, compare_binary_op, |a, b| a != 0 || b != 0))
            }

            Expression::LogicalNot(e) => {
                let mut evaluated = self.evaluate_value(e, symbol_resolution_callback)?;
                if self.current_section().is_none()
                    && let Some(sec) = evaluated.section_id()
                {
                    evaluated.value = evaluated
                        .value
                        .wrapping_add(self.section_layouts.get(sec).mem_offset);
                }
                evaluated.value_kind =
                    if self.current_section().is_some() && !evaluated.is_absolute() {
                        ExpressionValueKind::Number
                    } else {
                        ExpressionValueKind::Absolute
                    };
                evaluated.value = u64::from(evaluated.value == 0);
                Ok(evaluated)
            }
            Expression::BitwiseNot(e) => {
                let mut evaluated = self.evaluate_value(e, symbol_resolution_callback)?;
                evaluated.value = !evaluated.value;
                Ok(evaluated)
            }
            Expression::Negate(e) => {
                let mut evaluated = self.evaluate_value(e, symbol_resolution_callback)?;
                evaluated.value = evaluated.value.wrapping_neg();
                Ok(evaluated)
            }
            Expression::Origin(name) => {
                let region = self.memory_regions.get(name).with_context(|| {
                    format!(
                        "ORIGIN: memory region '{}' not found",
                        String::from_utf8_lossy(name)
                    )
                })?;
                if let Some(curr_sec) = self.current_section() {
                    let section_base = self.section_layouts.get(curr_sec).mem_offset;
                    Ok(EvaluatedValue {
                        value: region.origin.wrapping_sub(section_base),
                        value_kind: ExpressionValueKind::Relative(curr_sec),
                    })
                } else {
                    Ok(EvaluatedValue {
                        value: region.origin,
                        value_kind: ExpressionValueKind::Address,
                    })
                }
            }
            Expression::Length(name) => {
                let region = self.memory_regions.get(name).with_context(|| {
                    format!(
                        "LENGTH: memory region '{}' not found",
                        String::from_utf8_lossy(name)
                    )
                })?;
                Ok(EvaluatedValue::number(region.length))
            }
            Expression::SegmentStart(name, default_expr) => {
                let value = if let Some(value) = self.symbol_db.args.segment_start_override(*name) {
                    value
                } else {
                    self.evaluate_value(default_expr, symbol_resolution_callback)?
                        .value
                };
                if let Some(curr_sec) = self.current_section() {
                    let section_base = self.section_layouts.get(curr_sec).mem_offset;
                    Ok(EvaluatedValue {
                        value: value.wrapping_sub(section_base),
                        value_kind: ExpressionValueKind::Relative(curr_sec),
                    })
                } else {
                    Ok(EvaluatedValue {
                        value,
                        value_kind: ExpressionValueKind::Address,
                    })
                }
            }
            Expression::SizeofHeaders => Ok(EvaluatedValue::number(self.sizeof_headers)),
            Expression::Ternary(cond, if_true, if_false) => {
                let mut cond = self.evaluate_value(cond, symbol_resolution_callback)?;
                if self.current_section().is_none()
                    && let Some(sec) = cond.section_id()
                {
                    cond.value = cond
                        .value
                        .wrapping_add(self.section_layouts.get(sec).mem_offset);
                }
                self.evaluate_value(
                    if cond.value != 0 { if_true } else { if_false },
                    symbol_resolution_callback,
                )
            }
            Expression::Defined(name) => Ok(EvaluatedValue::number(
                self.symbol_db
                    .get_unversioned(&UnversionedSymbolName::prehashed(name))
                    .map_or(0, |_| 1),
            )),
            Expression::Assert(assert_command) => {
                let result =
                    self.evaluate_value(&assert_command.expression, symbol_resolution_callback)?;
                if result.value == 0 {
                    let msg = String::from_utf8_lossy(assert_command.message);
                    let Some(input_ref) = self.input_ref else {
                        bail!("{msg}");
                    };
                    let line = line_number(input_ref.data(), assert_command.remainder);
                    bail!("{}:{}: {msg}", input_ref, line);
                }
                Ok(result)
            }
            Expression::Absolute(e) => {
                let mut evaluated = self.evaluate_value(e, symbol_resolution_callback)?;
                if let Some(sec_id) = evaluated.section_id() {
                    evaluated.value = evaluated
                        .value
                        .wrapping_add(self.section_layouts.get(sec_id).mem_offset);
                }
                evaluated.value_kind = ExpressionValueKind::Absolute;
                Ok(evaluated)
            }
        }
    }
}

pub(crate) fn evaluate_const<'data>(expr: &Expression<'data>) -> Result<u64> {
    match expr {
        Expression::Number(n) => Ok(*n),
        Expression::Add(l, r) => Ok(evaluate_const(l)?.wrapping_add(evaluate_const(r)?)),
        Expression::Subtract(l, r) => Ok(evaluate_const(l)?.wrapping_sub(evaluate_const(r)?)),
        Expression::Multiply(l, r) => Ok(evaluate_const(l)?.wrapping_mul(evaluate_const(r)?)),
        Expression::Divide(l, r) => {
            let divisor = evaluate_const(r)?;
            if divisor == 0 {
                bail!("Division by zero in linker script expression");
            }
            Ok(((evaluate_const(l)? as i64).wrapping_div(divisor as i64)) as u64)
        }
        Expression::Modulo(l, r) => {
            let divisor = evaluate_const(r)?;
            if divisor == 0 {
                bail!("Modulo by zero in linker script expression");
            }
            Ok(((evaluate_const(l)? as i64).wrapping_rem(divisor as i64)) as u64)
        }
        Expression::LessThan(l, r) => Ok(u64::from(evaluate_const(l)? < evaluate_const(r)?)),
        Expression::GreaterThan(l, r) => Ok(u64::from(evaluate_const(l)? > evaluate_const(r)?)),
        Expression::LessEqual(l, r) => Ok(u64::from(evaluate_const(l)? <= evaluate_const(r)?)),
        Expression::GreaterEqual(l, r) => Ok(u64::from(evaluate_const(l)? >= evaluate_const(r)?)),
        Expression::Equal(l, r) => Ok(u64::from(evaluate_const(l)? == evaluate_const(r)?)),
        Expression::NotEqual(l, r) => Ok(u64::from(evaluate_const(l)? != evaluate_const(r)?)),
        Expression::Min(l, r) => Ok(evaluate_const(l)?.min(evaluate_const(r)?)),
        Expression::Max(l, r) => Ok(evaluate_const(l)?.max(evaluate_const(r)?)),
        Expression::BitwiseAnd(l, r) => Ok(evaluate_const(l)? & evaluate_const(r)?),
        Expression::BitwiseOr(l, r) => Ok(evaluate_const(l)? | evaluate_const(r)?),
        Expression::BitwiseXor(l, r) => Ok(evaluate_const(l)? ^ evaluate_const(r)?),
        Expression::LeftShift(l, r) => {
            Ok(evaluate_const(l)?.wrapping_shl(evaluate_const(r)? as u32))
        }
        Expression::RightShift(l, r) => {
            Ok(evaluate_const(l)?.wrapping_shr(evaluate_const(r)? as u32))
        }
        Expression::LogicalAnd(l, r) => Ok(u64::from(
            evaluate_const(l)? != 0 && evaluate_const(r)? != 0,
        )),
        Expression::LogicalOr(l, r) => Ok(u64::from(
            evaluate_const(l)? != 0 || evaluate_const(r)? != 0,
        )),
        Expression::LogicalNot(expression) => Ok(u64::from(evaluate_const(expression)? == 0)),
        Expression::BitwiseNot(expression) => Ok(!evaluate_const(expression)?),
        Expression::Negate(expression) => Ok(evaluate_const(expression)?.wrapping_neg()),
        Expression::Ternary(cond, if_true, if_false) => {
            let cond = evaluate_const(cond)?;
            if cond != 0 {
                evaluate_const(if_true)
            } else {
                evaluate_const(if_false)
            }
        }
        Expression::Absolute(expression) => evaluate_const(expression),

        _ => bail!("Expected constant expression"),
    }
}

pub(crate) fn evaluate_early_expression<'data, P: Platform>(
    expr: &Expression<'data>,
    loc: &SymbolLoc,
    memory_regions: &HashMap<&[u8], MemoryRegion>,
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    resolved_lc: &[ResolvedLocationCounter],
    laid_out_mem_offsets: &OutputSectionPartMap<Option<u64>>,
    group_states: &[GroupState<'data, P>],
    sizes: &OutputSectionPartMap<u64>,
    output_sections: &OutputSections<'data, P>,
    symbol_db: &SymbolDb<'data, P>,
    sizeof_headers: u64,
    section_positions: &OnceCell<InputSectionPositions>,
    visited_nodes: &mut hashbrown::HashSet<SymbolId>,
) -> Result<EvaluatedValue> {
    ExpressionContext {
        expr_loc: loc,
        input_ref: None,
        section_layouts,
        output_sections,
        memory_regions,
        symbol_db,
        sizeof_headers,
        resolved_location_counters: resolved_lc,
        laid_out_mem_offsets,
    }
    .evaluate(expr, &mut |name| {
        let Some(symbol_id) = symbol_db.get_unversioned(&UnversionedSymbolName::prehashed(name))
        else {
            bail!(
                "undefined symbol '{}' in linker script expression",
                String::from_utf8_lossy(name)
            );
        };

        let canonical_id = symbol_db.definition(symbol_id);
        let file_id = symbol_db.file_id_for_symbol(canonical_id);
        let file = group_states
            .get(file_id.group())
            .and_then(|group| group.files.get(file_id.file()));
        let def_info = match file {
            Some(FileLayoutState::Object(obj)) => {
                return layout::resolve_early_object_symbol(
                    canonical_id,
                    obj,
                    section_positions.get_or_init(|| {
                        layout::compute_input_section_positions(
                            group_states,
                            sizes.new_empty_like(),
                            symbol_db,
                            output_sections,
                        )
                    }),
                    symbol_db,
                );
            }
            Some(FileLayoutState::LinkerScript(ls))
                if let Group::LinkerScripts(scripts) = &symbol_db.groups[file_id.group()] =>
            {
                let script = &scripts[file_id.file()];
                let symbol_offset = ls.symbol_id_range.id_to_offset(canonical_id);
                &script.parsed.symbol_defs[symbol_offset]
            }
            Some(FileLayoutState::Prelude(prelude))
                if let Group::Prelude(p) = &symbol_db.groups[file_id.group()] =>
            {
                &p.symbol_definitions[prelude.symbol_id_range().id_to_offset(canonical_id)]
            }
            _ => {
                bail!(
                    "symbol '{}' is not defined by an input object",
                    symbol_db.symbol_name_for_display(canonical_id)
                );
            }
        };

        evaluate_early_expression_internal_symbol(
            memory_regions,
            section_layouts,
            resolved_lc,
            laid_out_mem_offsets,
            group_states,
            sizes,
            output_sections,
            symbol_db,
            sizeof_headers,
            section_positions,
            visited_nodes,
            canonical_id,
            def_info,
        )
    })
}

fn evaluate_early_expression_internal_symbol<'data, P: Platform>(
    memory_regions: &HashMap<&[u8], MemoryRegion>,
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    resolved_lc: &[ResolvedLocationCounter],
    laid_out_mem_offsets: &OutputSectionPartMap<Option<u64>>,
    group_states: &[GroupState<'data, P>],
    sizes: &OutputSectionPartMap<u64>,
    output_sections: &OutputSections<'data, P>,
    symbol_db: &SymbolDb<'data, P>,
    sizeof_headers: u64,
    section_positions: &OnceCell<InputSectionPositions>,
    visited_nodes: &mut hashbrown::HashSet<SymbolId>,
    canonical_id: SymbolId,
    def_info: &crate::parsing::InternalSymDefInfo<'data, P>,
) -> Result<SymbolValue> {
    match &def_info.placement {
        SymbolPlacement::Redirect(redirect) => {
            if !visited_nodes.insert(canonical_id) {
                return Ok(SymbolValue::Absolute(0));
            }
            let evaluated = evaluate_early_expression(
                &redirect.expression,
                &redirect.loc,
                memory_regions,
                section_layouts,
                resolved_lc,
                laid_out_mem_offsets,
                group_states,
                sizes,
                output_sections,
                symbol_db,
                sizeof_headers,
                section_positions,
                visited_nodes,
            )?;
            visited_nodes.remove(&canonical_id);
            let symbol_value = if evaluated.is_absolute() {
                SymbolValue::Absolute(evaluated.value)
            } else {
                SymbolValue::new(evaluated.value, evaluated.section_id())
            };
            Ok(symbol_value)
        }
        SymbolPlacement::SectionStart(section_id) => Ok(SymbolValue::SectionRelative {
            section_id: *section_id,
            address: 0,
        }),
        SymbolPlacement::SectionStartOffset(section_id, offset) => {
            Ok(SymbolValue::SectionRelative {
                section_id: *section_id,
                address: *offset,
            })
        }
        SymbolPlacement::SectionEnd(section_id) => Ok(SymbolValue::SectionRelative {
            section_id: *section_id,
            address: section_layouts.get(*section_id).mem_size,
        }),
        SymbolPlacement::TlsEnd(section_id) => {
            let mut end = 0;
            let mut alignment = crate::alignment::MIN;

            // Segment layouts aren't available until all sections have been laid out, so use
            // sections instead.
            for (id, info) in output_sections.ids_with_info() {
                if info.section_attributes.is_tls() {
                    let section = section_layouts.get(id);
                    end = end.max(section.mem_offset + section.mem_size);
                    alignment = alignment.max(section.alignment);
                }
            }

            Ok(SymbolValue::SectionRelative {
                section_id: *section_id,
                address: alignment.align_up(end),
            })
        }
        _ => {
            bail!("Unsupported symbol type");
        }
    }
}

fn section_size<'data, P: Platform>(
    name: &[u8],
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    output_sections: &OutputSections<'data, P>,
) -> u64 {
    // GNU ld returns 0 for SIZEOF of a section that doesn't exist in the output.
    // We match that behavior to avoid breaking scripts that guard with SIZEOF.
    let Some(id) = output_sections.section_id_by_name(SectionName(name)) else {
        return 0;
    };
    section_layouts.get(id).mem_size
}

fn section_align<'data, P: Platform>(
    name: &[u8],
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    output_sections: &OutputSections<'data, P>,
) -> u64 {
    // GNU ld returns 0 for ALIGNOF of a section that doesn't exist in the output.
    // We match that behavior to avoid breaking scripts that guard with SIZEOF.
    let Some(id) = output_sections.section_id_by_name(SectionName(name)) else {
        return 0;
    };
    section_layouts.get(id).alignment.value()
}

fn section_address<'data, P: Platform>(
    name: &[u8],
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    output_sections: &OutputSections<'data, P>,
) -> Result<EvaluatedValue> {
    let id = output_sections
        .section_id_by_name(SectionName(name))
        .with_context(|| {
            format!(
                "ADDR: section '{}' not found",
                String::from_utf8_lossy(name)
            )
        })?;
    let primary_section = output_sections.primary_output_section(id);
    let address = section_layouts.get(primary_section).mem_offset;
    evaluate_section_address(primary_section, address, section_layouts, name)
}

fn section_load_address<'data, P: Platform>(
    name: &[u8],
    section_layouts: &OutputSectionMap<OutputRecordLayout>,
    output_sections: &OutputSections<'data, P>,
) -> Result<u64> {
    let id = output_sections
        .section_id_by_name(SectionName(name))
        .with_context(|| {
            format!(
                "LOADADDR: section '{}' not found",
                String::from_utf8_lossy(name)
            )
        })?;
    Ok(section_layouts.get(id).lma_offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OsFileSystem;
    use crate::elf::Elf64;
    use crate::grouping::SequencedLinkerScript;
    use crate::input_data::FileId;
    use crate::layout::MemoryRegion;
    use crate::linker_script::AssertCommand;
    use crate::parsing::InternalSymDefInfo;
    use crate::parsing::ProcessedLinkerScript;
    use crate::parsing::Redirect;
    use crate::parsing::RedirectKind;
    use crate::parsing::SymbolPlacement;
    use crate::symbol_db::SymbolDb;
    use crate::symbol_db::SymbolIdRange;
    use colosseum::sync::Arena;

    fn with_dummy_context<R>(
        f: impl for<'test> FnOnce(
            &OutputSectionMap<OutputRecordLayout>,
            &OutputSections<'test, Elf64>,
            &mut SymbolDb<'test, Elf64>,
        ) -> R,
    ) -> R {
        let sections = OutputSections::<Elf64>::for_testing();
        let layouts = sections.new_section_map::<OutputRecordLayout>();
        let args = crate::args::elf::ElfArgs::new().unwrap();
        let output_kind = crate::output_kind::OutputKind::PartialLink;
        let arena = Arena::new();
        let auxiliary =
            crate::input_data::AuxiliaryFiles::new(&args, &[], &arena, &OsFileSystem).unwrap();
        let herd = Default::default();
        let mut symbol_db = SymbolDb::<Elf64>::new(&args, output_kind, &auxiliary, &herd).unwrap();
        f(&layouts, &sections, &mut symbol_db)
    }

    fn eval_const(expr: &Expression<'static>) -> Result<u64> {
        with_dummy_context(|layouts, sections, symbol_db| {
            ExpressionContext {
                expr_loc: &SymbolLoc::None,
                input_ref: None,
                section_layouts: layouts,
                output_sections: sections,
                memory_regions: &HashMap::new(),
                symbol_db,
                sizeof_headers: 0,
                resolved_location_counters: &[],
                laid_out_mem_offsets: &OutputSectionPartMap::default(),
            }
            .evaluate(expr, &mut |_| Ok(SymbolValue::Absolute(1)))
            .map(|evaluated| evaluated.value)
        })
    }

    #[test]
    fn test_number() {
        assert_eq!(eval_const(&Expression::Number(42)).unwrap(), 42);
        assert_eq!(eval_const(&Expression::Number(0)).unwrap(), 0);
    }

    #[test]
    fn test_arithmetic() {
        let add = Expression::Add(
            Box::new(Expression::Number(2)),
            Box::new(Expression::Number(3)),
        );
        assert_eq!(eval_const(&add).unwrap(), 5);

        let sub = Expression::Subtract(
            Box::new(Expression::Number(10)),
            Box::new(Expression::Number(4)),
        );
        assert_eq!(eval_const(&sub).unwrap(), 6);

        let mul = Expression::Multiply(
            Box::new(Expression::Number(3)),
            Box::new(Expression::Number(4)),
        );
        assert_eq!(eval_const(&mul).unwrap(), 12);

        let div = Expression::Divide(
            Box::new(Expression::Number(10)),
            Box::new(Expression::Number(2)),
        );
        assert_eq!(eval_const(&div).unwrap(), 5);
    }

    #[test]
    fn test_wrapping_arithmetic() {
        // u64::MAX + 1 should wrap to 0, not panic
        let expr = Expression::Add(
            Box::new(Expression::Number(u64::MAX)),
            Box::new(Expression::Number(1)),
        );
        assert_eq!(eval_const(&expr).unwrap(), 0);

        // 0 - 1 should wrap to u64::MAX
        let expr = Expression::Subtract(
            Box::new(Expression::Number(0)),
            Box::new(Expression::Number(1)),
        );
        assert_eq!(eval_const(&expr).unwrap(), u64::MAX);
    }

    #[test]
    fn test_operator_precedence() {
        // 1 + (2 * 3) = 7
        let expr = Expression::Add(
            Box::new(Expression::Number(1)),
            Box::new(Expression::Multiply(
                Box::new(Expression::Number(2)),
                Box::new(Expression::Number(3)),
            )),
        );
        assert_eq!(eval_const(&expr).unwrap(), 7);
    }

    #[test]
    fn test_comparisons() {
        // LessThan
        assert_eq!(
            eval_const(&Expression::LessThan(
                Box::new(Expression::Number(1)),
                Box::new(Expression::Number(2))
            ))
            .unwrap(),
            1
        );
        assert_eq!(
            eval_const(&Expression::LessThan(
                Box::new(Expression::Number(2)),
                Box::new(Expression::Number(1))
            ))
            .unwrap(),
            0
        );
        assert_eq!(
            eval_const(&Expression::LessThan(
                Box::new(Expression::Number(5)),
                Box::new(Expression::Number(5))
            ))
            .unwrap(),
            0
        );

        // GreaterThan
        assert_eq!(
            eval_const(&Expression::GreaterThan(
                Box::new(Expression::Number(3)),
                Box::new(Expression::Number(2))
            ))
            .unwrap(),
            1
        );
        assert_eq!(
            eval_const(&Expression::GreaterThan(
                Box::new(Expression::Number(2)),
                Box::new(Expression::Number(3))
            ))
            .unwrap(),
            0
        );
        assert_eq!(
            eval_const(&Expression::GreaterThan(
                Box::new(Expression::Number(5)),
                Box::new(Expression::Number(5))
            ))
            .unwrap(),
            0
        );

        // LessEqual
        assert_eq!(
            eval_const(&Expression::LessEqual(
                Box::new(Expression::Number(1)),
                Box::new(Expression::Number(2))
            ))
            .unwrap(),
            1
        );
        assert_eq!(
            eval_const(&Expression::LessEqual(
                Box::new(Expression::Number(5)),
                Box::new(Expression::Number(5))
            ))
            .unwrap(),
            1
        );
        assert_eq!(
            eval_const(&Expression::LessEqual(
                Box::new(Expression::Number(6)),
                Box::new(Expression::Number(5))
            ))
            .unwrap(),
            0
        );

        // GreaterEqual
        assert_eq!(
            eval_const(&Expression::GreaterEqual(
                Box::new(Expression::Number(5)),
                Box::new(Expression::Number(5))
            ))
            .unwrap(),
            1
        );
        assert_eq!(
            eval_const(&Expression::GreaterEqual(
                Box::new(Expression::Number(6)),
                Box::new(Expression::Number(5))
            ))
            .unwrap(),
            1
        );
        assert_eq!(
            eval_const(&Expression::GreaterEqual(
                Box::new(Expression::Number(4)),
                Box::new(Expression::Number(5))
            ))
            .unwrap(),
            0
        );

        // Equal / NotEqual
        assert_eq!(
            eval_const(&Expression::Equal(
                Box::new(Expression::Number(5)),
                Box::new(Expression::Number(5))
            ))
            .unwrap(),
            1
        );
        assert_eq!(
            eval_const(&Expression::Equal(
                Box::new(Expression::Number(5)),
                Box::new(Expression::Number(6))
            ))
            .unwrap(),
            0
        );
        assert_eq!(
            eval_const(&Expression::NotEqual(
                Box::new(Expression::Number(5)),
                Box::new(Expression::Number(6))
            ))
            .unwrap(),
            1
        );
        assert_eq!(
            eval_const(&Expression::NotEqual(
                Box::new(Expression::Number(5)),
                Box::new(Expression::Number(5))
            ))
            .unwrap(),
            0
        );
    }

    #[test]
    fn test_min_max() {
        assert_eq!(
            eval_const(&Expression::Min(
                Box::new(Expression::Number(3)),
                Box::new(Expression::Number(7))
            ))
            .unwrap(),
            3
        );
        assert_eq!(
            eval_const(&Expression::Min(
                Box::new(Expression::Number(7)),
                Box::new(Expression::Number(3))
            ))
            .unwrap(),
            3
        );
        assert_eq!(
            eval_const(&Expression::Max(
                Box::new(Expression::Number(3)),
                Box::new(Expression::Number(7))
            ))
            .unwrap(),
            7
        );
        assert_eq!(
            eval_const(&Expression::Max(
                Box::new(Expression::Number(7)),
                Box::new(Expression::Number(3))
            ))
            .unwrap(),
            7
        );
        // equal values
        assert_eq!(
            eval_const(&Expression::Min(
                Box::new(Expression::Number(5)),
                Box::new(Expression::Number(5))
            ))
            .unwrap(),
            5
        );
        assert_eq!(
            eval_const(&Expression::Max(
                Box::new(Expression::Number(5)),
                Box::new(Expression::Number(5))
            ))
            .unwrap(),
            5
        );
    }

    #[test]
    fn test_align() {
        // ALIGN(8) with location counter 0 → 0
        assert_eq!(
            eval_const(&Expression::Align(Box::new(Expression::Number(8)), None)).unwrap(),
            0
        );
        // ALIGN(1) → 0
        assert_eq!(
            eval_const(&Expression::Align(Box::new(Expression::Number(1)), None)).unwrap(),
            0
        );
    }

    #[test]
    fn test_align_zero_is_error() {
        assert!(eval_const(&Expression::Align(Box::new(Expression::Number(0)), None)).is_err());
    }

    #[test]
    fn test_divide_by_zero() {
        let expr = Expression::Divide(
            Box::new(Expression::Number(10)),
            Box::new(Expression::Number(0)),
        );
        assert!(eval_const(&expr).is_err());
    }

    #[test]
    fn test_modulo_by_zero() {
        let expr = Expression::Modulo(
            Box::new(Expression::Number(10)),
            Box::new(Expression::Number(0)),
        );
        assert!(eval_const(&expr).is_err());
    }

    #[test]
    fn test_location_counter_is_zero() {
        // LocationCounter outside a section context is treated as 0
        assert_eq!(eval_const(&Expression::LocationCounter).unwrap(), 0);
    }

    #[test]
    fn test_alignof_evaluation() {
        // Test that evaluating ALIGNOF for a non-existent section returns 0
        assert_eq!(
            eval_const(&Expression::Alignof(b".nonexistent")).unwrap(),
            0
        );
    }

    fn make_script<'data>(
        assertions: &[AssertCommand<'static>],
    ) -> SequencedLinkerScript<'data, Elf64> {
        SequencedLinkerScript {
            parsed: ProcessedLinkerScript {
                input: crate::input_data::InputRef {
                    file: crate::input_data::InputFileRef::for_testing(),
                    data: &[],
                    entry: None,
                },
                symbol_defs: assertions
                    .iter()
                    .map(|assertion| {
                        InternalSymDefInfo::new(
                            SymbolPlacement::Redirect(Redirect {
                                kind: RedirectKind::Script,
                                expression: Expression::Assert(assertion.clone()),
                                loc: SymbolLoc::None,
                            }),
                            b"",
                        )
                    })
                    .collect(),
                memory_regions: Vec::new(),
                program_headers: Vec::new(),
                location_counters: Vec::new(),
                ordered_sections: Vec::new(),
            },
            symbol_id_range: SymbolIdRange::empty(),
            file_id: FileId::new(0, 0),
        }
    }

    fn evaluate_assertions<'data>(
        script: &SequencedLinkerScript<'data, Elf64>,
        symbol_db: &SymbolDb<'data, Elf64>,
        section_layouts: &OutputSectionMap<OutputRecordLayout>,
        output_sections: &OutputSections<'data, Elf64>,
        sizeof_headers: u64,
        memory_regions: &HashMap<&[u8], layout::MemoryRegion>,
        resolved_location_counters: &[ResolvedLocationCounter],
    ) -> Result {
        for assertion in &script.parsed.symbol_defs {
            let SymbolPlacement::Redirect(redirect) = &assertion.placement else {
                continue;
            };
            ExpressionContext {
                expr_loc: &SymbolLoc::None,
                input_ref: None,
                section_layouts,
                output_sections,
                memory_regions,
                symbol_db,
                sizeof_headers,
                resolved_location_counters,
                laid_out_mem_offsets: &OutputSectionPartMap::default(),
            }
            .evaluate(&redirect.expression, &mut |_| unreachable!())?;
        }
        Ok(())
    }

    #[test]
    fn test_evaluate_assertions_passes() {
        with_dummy_context(|layouts, sections, symbol_db| {
            let script = make_script(&[AssertCommand {
                expression: Box::new(Expression::Equal(
                    Box::new(Expression::Number(1)),
                    Box::new(Expression::Number(1)),
                )),
                message: b"should pass",
                remainder: b"",
            }]);
            assert!(
                evaluate_assertions(
                    &script,
                    symbol_db,
                    layouts,
                    sections,
                    0,
                    &HashMap::new(),
                    &[]
                )
                .is_ok()
            );
        });
    }

    #[test]
    fn test_evaluate_assertions_fails() {
        with_dummy_context(|layouts, sections, symbol_db| {
            let script = make_script(&[AssertCommand {
                expression: Box::new(Expression::Number(0)),
                message: b"intentional failure",
                remainder: b"",
            }]);
            let err = evaluate_assertions(
                &script,
                symbol_db,
                layouts,
                sections,
                0,
                &HashMap::new(),
                &[],
            )
            .unwrap_err();
            assert!(err.to_string().contains("intentional failure"));
        });
    }

    #[test]
    fn test_memory_functions_evaluation() {
        with_dummy_context(|layouts, sections, symbol_db| {
            let regions = HashMap::from([
                (
                    b"rom" as &[u8],
                    MemoryRegion {
                        origin: 0x08000000,
                        length: 0x100000,
                        used: 0,
                    },
                ),
                (
                    b"ram" as &[u8],
                    MemoryRegion {
                        origin: 0x20000000,
                        length: 0x40000,
                        used: 0,
                    },
                ),
            ]);
            let eval = |expr: &Expression<'static>| {
                ExpressionContext {
                    expr_loc: &SymbolLoc::None,
                    input_ref: None,
                    section_layouts: layouts,
                    output_sections: sections,
                    memory_regions: &regions,
                    symbol_db,
                    sizeof_headers: 0,
                    resolved_location_counters: &[],
                    laid_out_mem_offsets: &OutputSectionPartMap::default(),
                }
                .evaluate(expr, &mut |_| Ok(SymbolValue::Absolute(0)))
                .map(|evaluated| evaluated.value)
            };
            assert_eq!(eval(&Expression::Origin(b"rom")).unwrap(), 0x08000000);
            assert_eq!(eval(&Expression::Length(b"rom")).unwrap(), 0x100000);
            assert_eq!(eval(&Expression::Origin(b"ram")).unwrap(), 0x20000000);
            assert_eq!(eval(&Expression::Length(b"ram")).unwrap(), 0x40000);
            // end of rom = origin + length
            let end = Expression::Add(
                Box::new(Expression::Origin(b"rom")),
                Box::new(Expression::Length(b"rom")),
            );
            assert_eq!(eval(&end).unwrap(), 0x08100000);
            assert!(eval(&Expression::Origin(b"flash")).is_err());
        });
    }
}
