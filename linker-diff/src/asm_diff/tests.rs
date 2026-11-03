use super::*;
use crate::aarch64::AArch64;
use crate::aarch64::RType as AArch64RelocationType;
use crate::x86_64::RType as X86RelocationType;
use crate::x86_64::X86_64;
use linker_utils::aarch64::RelaxationKind as AArch64RelaxationKind;
use linker_utils::x86_64::RelaxationKind as X86RelaxationKind;

fn named(name: &'static [u8]) -> Referent<'static, X86RelocationType> {
    Referent::Named(
        SymbolName {
            bytes: name,
            version: None,
        },
        0,
    )
}

fn resolved(
    referent: Referent<'static, X86RelocationType>,
    indirection: Indirection,
    relaxation_kind: X86RelaxationKind,
) -> ResolvedGroup<'static, X86_64> {
    let relaxation = RelaxationMatch {
        relaxation: Relaxation {
            relaxation_kind,
            new_r_type: X86RelocationType::from_raw(object::elf::R_X86_64_PC32),
            alt_r_type: None,
        },
        value: 0,
        start: 0,
        end: 4,
        original_referent: named(b"target"),
        addend: 0,
        offset: 0,
        next_modifier: RelocationModifier::Normal,
    };
    ResolvedGroup {
        relaxations: Some(RelaxationGroup {
            match_results: vec![RelaxationMatchResult::Matched(relaxation)],
            is_complete: true,
        }),
        failure: None,
        annotations: vec![relaxation.annotation()],
        reference: Reference {
            referent,
            indirection,
        },
        start: 0,
        end: 4,
        trace: TraceOutput::default(),
    }
}

fn key(comparison: &ResolutionComparison<X86_64>) -> String {
    comparison_key(
        comparison,
        &[X86RelocationType::from_raw(object::elf::R_X86_64_PC32)],
    )
}

fn comparison_key<A: Arch>(
    comparison: &ResolutionComparison<A>,
    input_types: &[A::RType],
) -> String {
    let annotations = input_types
        .iter()
        .enumerate()
        .map(|(index, &r_type)| OriginalAnnotation {
            success: MatchedRelaxation {
                r_type,
                relaxation_kind: None,
            },
            reference: Reference::unknown(),
            trace: TraceOutput::default(),
            offset: index as u64 * 4,
        })
        .collect_vec();
    comparison.diff_key(
        &annotations,
        BinAttributes {
            output_kind: OutputKind::Executable,
            relocatability: Relocatability::NonRelocatable,
            link_type: LinkType::Static,
        },
    )
}

#[test]
fn selects_the_reference_that_differs() {
    let ours = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let matching = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let differing = resolved(
        named(b"target"),
        Indirection::PltGot,
        X86RelaxationKind::NoOp,
    );
    let comparisons = [ours.compare(&matching), ours.compare(&differing)];

    let (index, comparison) = select_comparison(&comparisons, false, |_, _| false).unwrap();
    assert_eq!(index, 1);
    assert_eq!(key(comparison), "rel.missing-plt-got");
    assert!(select_comparison(&comparisons, true, |_, _| false).is_none());

    let comparisons = [ours.compare(&differing), ours.compare(&matching)];
    assert!(select_comparison(&comparisons, true, |_, _| false).is_none());
    assert_eq!(
        select_comparison(&comparisons, false, |_, _| false)
            .unwrap()
            .0,
        0
    );
}

#[test]
fn ignored_tls_relaxation_does_not_hide_a_later_wrong_offset() {
    let mut config = crate::Config::default();
    config.apply_wild_defaults(crate::arch::ArchKind::X86_64);
    let report = Report {
        names: vec![],
        paths: vec![],
        diffs: vec![],
        config,
        coverage: None,
    };
    let ours = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::RexAddIndirectToAbsolute(3),
    );
    let good = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::RexAddIndirectToLea(3),
    );
    let bad = resolved(
        Referent::Named(
            SymbolName {
                bytes: b"target",
                version: None,
            },
            1,
        ),
        Indirection::Direct,
        X86RelaxationKind::RexAddIndirectToLea(3),
    );
    let input_types = [X86RelocationType::from_raw(object::elf::R_X86_64_GOTTPOFF)];
    let mut comparisons = [ours.compare(&good), ours.compare(&bad)];
    let key = |comparison: &ResolutionComparison<X86_64>| comparison_key(comparison, &input_types);
    assert!(report.should_ignore(&key(&comparisons[0])));
    assert!(!report.should_ignore(&key(&comparisons[1])));
    for expected_index in [1, 0] {
        let (index, comparison) = select_comparison(&comparisons, false, |_, comparison| {
            report.should_ignore(&key(comparison))
        })
        .unwrap();
        assert!(!report.should_ignore(&key(comparison)));
        assert_eq!(index, expected_index);
        assert_eq!(key(comparison), "rel.referent-mismatch");
        let (_, comparison) = select_comparison(&comparisons, true, |_, comparison| {
            report.should_ignore(&key(comparison))
        })
        .unwrap();
        assert!(report.should_ignore(&key(comparison)));
        comparisons.swap(0, 1);
    }
}

#[test]
fn ignored_resolution_failure_does_not_hide_a_reference_difference() {
    let ours = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let bad = resolved(
        named(b"wrong_target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let mut comparisons = [
        ResolutionComparison::Unresolved(ResolutionFailure::MatchFailed(
            X86RelocationType::from_raw(object::elf::R_X86_64_PC32),
        )),
        ours.compare(&bad),
    ];
    for expected_index in [1, 0] {
        let (index, comparison) = select_comparison(&comparisons, false, |_, comparison| {
            key(comparison) == "rel.match_failed.R_X86_64_PC32"
        })
        .unwrap();
        assert_eq!(index, expected_index);
        assert_eq!(key(comparison), "rel.referent-mismatch");
        comparisons.swap(0, 1);
    }
    let (_, comparison) = select_comparison(&comparisons[..1], false, |_, _| true).unwrap();
    assert!(matches!(comparison, ResolutionComparison::Unresolved(_)));
}

#[test]
fn retains_simultaneous_differences() {
    let ours = resolved(named(b"target"), Indirection::Got, X86RelaxationKind::NoOp);
    let other = resolved(
        named(b"wrong_target"),
        Indirection::Direct,
        X86RelaxationKind::MovIndirectToLea,
    );
    let comparison = ours.compare(&other);
    let ResolutionComparison::Different(diff) = &comparison else {
        panic!("Expected differences, got {comparison:?}");
    };
    assert!(matches!(
        diff.relaxation.as_ref().map(|diff| &diff.kind),
        Some(RelaxationDifferenceKind::MissingOptimization(_))
    ));
    assert!(matches!(
        diff.reference.referent,
        Some(ReferentDifference::Other)
    ));
    assert!(matches!(
        diff.reference.indirection,
        Some(IndirectionDifference::Other)
    ));
    assert_eq!(key(&comparison), "rel.referent-mismatch");
}

#[test]
fn optimization_ignores_do_not_hide_referent_mismatches() {
    let report = Report {
        names: vec![],
        paths: vec![],
        diffs: vec![],
        config: crate::Config {
            ignore: vec!["rel.missing-opt.*".to_owned(), "rel.extra-opt.*".to_owned()],
            ..Default::default()
        },
        coverage: None,
    };
    let ours = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let mut other = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::MovIndirectToLea,
    );

    for (comparison, expected) in [
        (
            ours.compare(&other),
            "rel.missing-opt.R_X86_64_PC32.MovIndirectToLea.static-non-pie",
        ),
        (
            other.compare(&ours),
            "rel.extra-opt.R_X86_64_PC32.MovIndirectToLea.static-non-pie",
        ),
    ] {
        let key = key(&comparison);
        assert_eq!(key, expected);
        assert!(report.should_ignore(&key));
    }

    other.reference.referent = named(b"wrong_target");
    for comparison in [ours.compare(&other), other.compare(&ours)] {
        let key = key(&comparison);
        assert_eq!(key, "rel.referent-mismatch");
        assert!(!report.should_ignore(&key));
    }
}

#[test]
fn absolute_relaxation_requires_the_same_value() {
    let ours = resolved(
        Referent::Undefined(UnmatchedAddress {
            address: 42,
            reason: "",
        }),
        Indirection::Direct,
        X86RelaxationKind::RexMovIndirectToAbsolute(3),
    );
    for value in [42, 43] {
        let other = resolved(
            Referent::Absolute(value),
            Indirection::Got,
            X86RelaxationKind::NoOp,
        );
        for (comparison, direction) in [
            (ours.compare(&other), "extra"),
            (other.compare(&ours), "missing"),
        ] {
            let expected = if value == 42 {
                format!(
                    "rel.{direction}-opt.R_X86_64_PC32.RexMovIndirectToAbsolute(3).static-non-pie"
                )
            } else {
                "rel.referent-mismatch".to_owned()
            };
            assert_eq!(key(&comparison), expected);
        }
    }
}

#[test]
fn tls_model_relaxation_requires_the_same_symbol_and_addend() {
    let entry = SymtabEntryInfo {
        name: SymbolName {
            bytes: b"target",
            version: None,
        },
        is_weak: false,
        visibility: Visibility::Default,
        is_ifunc: false,
    };
    let ours = resolved(
        Referent::TlsGd(entry),
        Indirection::Got,
        X86RelaxationKind::NoOp,
    );
    for (name, addend) in [
        (b"target".as_slice(), 0),
        (b"wrong_target".as_slice(), 0),
        (b"target".as_slice(), 1),
    ] {
        let other = resolved(
            Referent::DynamicRelocation(DynamicRelocation {
                entry: SymtabEntryInfo {
                    name: SymbolName {
                        bytes: name,
                        version: None,
                    },
                    ..entry
                },
                r_type: X86RelocationType::from_dynamic_relocation_kind(
                    DynamicRelocationKind::TpOff,
                ),
                addend,
            }),
            Indirection::Got,
            X86RelaxationKind::TlsGdToInitialExec,
        );
        for (comparison, direction) in [
            (ours.compare(&other), "missing"),
            (other.compare(&ours), "extra"),
        ] {
            let expected = if name == b"target" && addend == 0 {
                format!("rel.{direction}-opt.R_X86_64_PC32.TlsGdToInitialExec.static-non-pie")
            } else {
                "rel.referent-mismatch".to_owned()
            };
            assert_eq!(key(&comparison), expected);
        }
    }
}

#[test]
fn indirection_key_takes_precedence_over_relaxation_key() {
    let ours = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let other = resolved(
        named(b"target"),
        Indirection::PltGot,
        X86RelaxationKind::MovIndirectToLea,
    );
    assert_eq!(key(&ours.compare(&other)), "rel.missing-plt-got");
    assert_eq!(key(&other.compare(&ours)), "rel.extra-plt-got");
}

#[test]
fn unresolved_outputs_do_not_match_each_other() {
    let mut ours = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let mut other = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    ours.relaxations = None;
    other.relaxations = None;
    let comparisons = [ours.compare(&other)];
    assert!(matches!(
        comparisons[0],
        ResolutionComparison::Unresolved(_)
    ));
    assert!(select_comparison(&comparisons, true, |_, _| false).is_some());
    assert_eq!(key(&comparisons[0]), "rel.unknown_failure");
}

#[test]
fn resolution_errors_are_not_equivalent() {
    let mut ours = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let other = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    ours.failure = Some(ResolutionFailure::Reference(X86RelocationType::from_raw(
        object::elf::R_X86_64_PC32,
    )));
    ours.annotations.push(Annotation {
        offset_in_section: 4,
        kind: AnnotationKind::Error("Unable to read reference".to_owned()),
    });
    let comparison = ours.compare(&other);
    assert!(matches!(comparison, ResolutionComparison::Unresolved(_)));
    assert_eq!(key(&comparison), "rel.reference_failed.R_X86_64_PC32");
    assert!(select_comparison(&[comparison], true, |_, _| false).is_some());
}

#[test]
fn preserves_dynamic_relocation_equivalence_and_classifies_plt_bypass() {
    let entry = SymtabEntryInfo {
        name: SymbolName {
            bytes: b"target",
            version: None,
        },
        is_weak: false,
        visibility: Visibility::Default,
        is_ifunc: false,
    };
    let dynamic = |kind| {
        Referent::DynamicRelocation(DynamicRelocation {
            entry,
            r_type: X86RelocationType::from_dynamic_relocation_kind(kind),
            addend: 0,
        })
    };
    let ours = resolved(
        dynamic(DynamicRelocationKind::GotEntry),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let mut other = resolved(
        dynamic(DynamicRelocationKind::JumpSlot),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    assert!(matches!(
        ours.compare(&other),
        ResolutionComparison::Equivalent
    ));
    other.reference.indirection = Indirection::PltGot;
    assert_eq!(key(&ours.compare(&other)), "rel.dynamic-plt-bypass");

    let ours = resolved(
        dynamic(DynamicRelocationKind::Absolute),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let comparison = ours.compare(&other);
    let ResolutionComparison::Different(diff) = &comparison else {
        panic!("Expected differences, got {comparison:?}");
    };
    assert!(matches!(
        diff.reference.referent,
        Some(ReferentDifference::Other)
    ));
    assert_eq!(key(&comparison), "rel.dynamic-plt-bypass");
}

#[test]
fn copy_relocation_classification_uses_the_differing_pair() {
    let target = SymbolName {
        bytes: b"target",
        version: None,
    };
    let ours = resolved(
        Referent::Copy(target, 0),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let other = resolved(
        Referent::Named(target, 0),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let comparisons = [ours.compare(&ours), ours.compare(&other)];
    let (index, comparison) = select_comparison(&comparisons, false, |_, _| false).unwrap();
    assert_eq!(index, 1);
    assert_eq!(key(comparison), "rel.extra-copy-relocation.R_X86_64_PC32");
    assert_eq!(
        key(&other.compare(&ours)),
        "rel.missing-copy-relocation.R_X86_64_PC32"
    );
}

fn matched(r_type: object::elf::RelocationType) -> RelaxationMatch<'static, X86_64> {
    RelaxationMatch {
        relaxation: Relaxation {
            relaxation_kind: X86RelaxationKind::NoOp,
            new_r_type: X86RelocationType::from_raw(r_type),
            alt_r_type: None,
        },
        value: 0,
        start: 0,
        end: 4,
        original_referent: Referent::Unknown,
        addend: 0,
        offset: 0,
        next_modifier: RelocationModifier::Normal,
    }
}

#[test]
fn failure_without_candidate_annotations_keeps_its_input_relocation_type() {
    let group = RelaxationGroup::<X86_64> {
        match_results: vec![
            RelaxationMatchResult::Matched(matched(object::elf::R_X86_64_PC32)),
            RelaxationMatchResult::AllFailed(vec![]),
        ],
        is_complete: true,
    };
    let r_types = [object::elf::R_X86_64_GOTPCRELX, object::elf::R_X86_64_PLT32]
        .map(X86RelocationType::from_raw);

    assert!(group.match_results[1].annotations().is_empty());
    assert_eq!(
        group.failure(r_types.into_iter()).unwrap().diff_key(),
        "rel.match_failed.R_X86_64_PLT32"
    );
}

#[test]
fn failure_with_no_candidates_is_not_lost_before_an_ambiguous_match() {
    let candidate = matched(object::elf::R_X86_64_PC32);
    let group = RelaxationGroup::<X86_64> {
        match_results: vec![
            RelaxationMatchResult::AllFailed(vec![]),
            RelaxationMatchResult::Ambiguous(vec![candidate, candidate]),
        ],
        is_complete: true,
    };
    let r_types = [object::elf::R_X86_64_PLT32, object::elf::R_X86_64_GOTPCRELX]
        .map(X86RelocationType::from_raw);

    assert_eq!(
        group.failure(r_types.into_iter()).unwrap().diff_key(),
        "rel.match_failed.R_X86_64_PLT32"
    );
}

#[test]
fn reference_failure_keys_do_not_depend_on_diagnostic_text() {
    let r_type = X86RelocationType::from_raw(object::elf::R_X86_64_TLSGD);
    for message in [
        "Missing dynamic relocation for TlsGd",
        "Unable to read GOT at 0x1234",
    ] {
        let candidate = matched(object::elf::R_X86_64_TLSGD);
        let resolution = ResolvedGroup {
            relaxations: Some(RelaxationGroup {
                match_results: vec![RelaxationMatchResult::Matched(candidate)],
                is_complete: true,
            }),
            failure: Some(ResolutionFailure::Reference(r_type)),
            annotations: vec![
                candidate.annotation(),
                Annotation {
                    offset_in_section: 4,
                    kind: AnnotationKind::Error(message.to_owned()),
                },
            ],
            reference: Reference::unknown(),
            start: 0,
            end: 4,
            trace: TraceOutput::default(),
        };
        let reference = ResolvedGroup {
            failure: None,
            annotations: vec![candidate.annotation()],
            relaxations: Some(RelaxationGroup {
                match_results: vec![RelaxationMatchResult::Matched(candidate)],
                is_complete: true,
            }),
            reference: Reference::unknown(),
            start: 0,
            end: 4,
            trace: TraceOutput::default(),
        };
        assert!(matches!(
            resolution.failure,
            Some(ResolutionFailure::Reference(_))
        ));
        assert_eq!(
            resolution.compare(&reference).diff_key(
                &[],
                BinAttributes {
                    output_kind: OutputKind::Executable,
                    relocatability: Relocatability::Relocatable,
                    link_type: LinkType::Dynamic,
                }
            ),
            "rel.reference_failed.R_X86_64_TLSGD"
        );
    }
}

#[test]
fn reference_failure_keys_do_not_depend_on_reference_order() {
    let r_type = X86RelocationType::from_raw(object::elf::R_X86_64_TLSGD);
    let candidate = matched(object::elf::R_X86_64_TLSGD);
    let original_annotations = [OriginalAnnotation {
        success: MatchedRelaxation {
            r_type,
            relaxation_kind: None,
        },
        reference: Reference::unknown(),
        trace: TraceOutput::default(),
        offset: 0,
    }];
    let bin_attributes = BinAttributes {
        output_kind: OutputKind::Executable,
        relocatability: Relocatability::Relocatable,
        link_type: LinkType::Dynamic,
    };
    let mut resolutions = [false, false, true].map(|failed| ResolvedGroup {
        relaxations: Some(RelaxationGroup {
            match_results: vec![RelaxationMatchResult::Matched(candidate)],
            is_complete: true,
        }),
        failure: failed.then_some(ResolutionFailure::Reference(r_type)),
        annotations: vec![candidate.annotation()],
        reference: if failed {
            Reference::unknown()
        } else {
            Reference::direct(Referent::Absolute(42))
        },
        start: 0,
        end: 4,
        trace: TraceOutput::default(),
    });

    assert!(matches!(
        resolutions[0].compare(&resolutions[1]),
        ResolutionComparison::Equivalent
    ));
    assert!(matches!(
        resolutions[0].compare(&resolutions[2]),
        ResolutionComparison::Unresolved(_)
    ));
    for _ in 0..2 {
        let comparisons = resolutions[1..]
            .iter()
            .map(|reference| resolutions[0].compare(reference))
            .collect_vec();
        let (_, comparison) = select_comparison(&comparisons, false, |_, _| false).unwrap();
        assert_eq!(
            comparison.diff_key(&original_annotations, bin_attributes),
            "rel.reference_failed.R_X86_64_TLSGD"
        );
        assert!(select_comparison(&comparisons, true, |_, _| false).is_none());
        resolutions.swap(1, 2);
    }

    resolutions[0].failure = Some(ResolutionFailure::Ambiguous);
    let comparisons = resolutions[1..]
        .iter()
        .map(|reference| resolutions[0].compare(reference))
        .collect_vec();
    for match_any in [false, true] {
        let (_, comparison) = select_comparison(&comparisons, match_any, |_, _| false).unwrap();
        assert_eq!(
            comparison.diff_key(&original_annotations, bin_attributes),
            "rel.multiple_matches"
        );
    }
}

#[test]
fn resolution_failures_take_precedence_over_optimization_differences() {
    let ours = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::MovIndirectToLea,
    );
    let differing = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let mut failed = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    failed.failure = Some(ResolutionFailure::Reference(X86RelocationType::from_raw(
        object::elf::R_X86_64_PC32,
    )));
    let mut comparisons = vec![ours.compare(&differing), ours.compare(&failed)];
    assert_eq!(
        key(&comparisons[0]),
        "rel.extra-opt.R_X86_64_PC32.MovIndirectToLea.static-non-pie"
    );

    for expected_index in [1, 0] {
        for match_any in [false, true] {
            let (index, comparison) =
                select_comparison(&comparisons, match_any, |_, _| false).unwrap();
            assert_eq!(index, expected_index);
            assert_eq!(key(comparison), "rel.reference_failed.R_X86_64_PC32");
        }
        comparisons.swap(0, 1);
    }

    comparisons.push(ours.compare(&ours));
    assert!(select_comparison(&comparisons, true, |_, _| false).is_none());
    assert_eq!(
        key(select_comparison(&comparisons, false, |_, _| false)
            .unwrap()
            .1),
        "rel.reference_failed.R_X86_64_PC32"
    );
}

#[test]
fn classifies_the_later_relocation_that_differs() {
    let mut ours = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let mut reference = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    ours.relaxations
        .as_mut()
        .unwrap()
        .match_results
        .push(RelaxationMatchResult::Matched(matched(
            object::elf::R_X86_64_GOTPCRELX,
        )));
    let mut optimized = matched(object::elf::R_X86_64_PC32);
    optimized.relaxation.relaxation_kind = X86RelaxationKind::MovIndirectToLea;
    reference
        .relaxations
        .as_mut()
        .unwrap()
        .match_results
        .push(RelaxationMatchResult::Matched(optimized));

    let comparison = ours.compare(&reference);
    let ResolutionComparison::Different(diff) = &comparison else {
        panic!("Expected a difference, got {comparison:?}");
    };
    assert_eq!(
        diff.relaxation.as_ref().unwrap().input_indexes,
        [Some(1), Some(1)]
    );
    let input_types = [object::elf::R_X86_64_PLT32, object::elf::R_X86_64_GOTPCRELX]
        .map(X86RelocationType::from_raw);
    assert_eq!(
        comparison_key(&comparison, &input_types),
        "rel.missing-opt.R_X86_64_GOTPCRELX.MovIndirectToLea.static-non-pie"
    );
    assert_eq!(
        comparison_key(&reference.compare(&ours), &input_types),
        "rel.extra-opt.R_X86_64_GOTPCRELX.MovIndirectToLea.static-non-pie"
    );
}

#[test]
fn relocation_type_key_uses_the_later_difference() {
    let mut ours = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let mut reference = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    ours.relaxations
        .as_mut()
        .unwrap()
        .match_results
        .push(RelaxationMatchResult::Matched(matched(
            object::elf::R_X86_64_64,
        )));
    reference
        .relaxations
        .as_mut()
        .unwrap()
        .match_results
        .push(RelaxationMatchResult::Matched(matched(
            object::elf::R_X86_64_32S,
        )));
    let input_types =
        [object::elf::R_X86_64_PC32, object::elf::R_X86_64_64].map(X86RelocationType::from_raw);
    assert_eq!(
        comparison_key(&ours.compare(&reference), &input_types),
        "rel.R_X86_64_64.R_X86_64_32S"
    );
}

#[test]
fn unequal_group_lengths_identify_the_unpaired_relocation() {
    let ours = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    let mut reference = resolved(
        named(b"target"),
        Indirection::Direct,
        X86RelaxationKind::NoOp,
    );
    reference
        .relaxations
        .as_mut()
        .unwrap()
        .match_results
        .push(RelaxationMatchResult::Matched(matched(
            object::elf::R_X86_64_PLT32,
        )));
    let input_types =
        [object::elf::R_X86_64_PC32, object::elf::R_X86_64_PLT32].map(X86RelocationType::from_raw);
    assert_eq!(
        comparison_key(&ours.compare(&reference), &input_types),
        "rel.missing-relocation.R_X86_64_PLT32"
    );
    assert_eq!(
        comparison_key(&reference.compare(&ours), &input_types),
        "rel.extra-relocation.R_X86_64_PLT32"
    );
}

fn aarch64_group(kinds: &[AArch64RelaxationKind]) -> ResolvedGroup<'static, AArch64> {
    let match_results = kinds
        .iter()
        .enumerate()
        .map(|(index, &kind)| {
            let r_type = if kind.is_replace_with_no_op() {
                object::elf::R_AARCH64_NONE
            } else {
                object::elf::R_AARCH64_ADR_PREL_PG_HI21
            };
            RelaxationMatchResult::Matched(RelaxationMatch {
                relaxation: Relaxation {
                    relaxation_kind: kind,
                    new_r_type: AArch64RelocationType::from_raw(r_type),
                    alt_r_type: None,
                },
                value: 0,
                start: index as u64 * 4,
                end: (index as u64 + 1) * 4,
                original_referent: Referent::Unknown,
                addend: 0,
                offset: index as u64 * 4,
                next_modifier: RelocationModifier::Normal,
            })
        })
        .collect();
    ResolvedGroup {
        relaxations: Some(RelaxationGroup {
            match_results,
            is_complete: true,
        }),
        failure: None,
        annotations: vec![],
        reference: Reference::unknown(),
        start: 0,
        end: kinds.len() as u64 * 4,
        trace: TraceOutput::default(),
    }
}

#[test]
fn removing_an_undefined_reference_keeps_the_optimization_key() {
    let mut ours = aarch64_group(&[AArch64RelaxationKind::NoOp]);
    ours.reference.referent = Referent::Undefined(UnmatchedAddress::default());
    let mut other = aarch64_group(&[AArch64RelaxationKind::ReplaceWithNop]);
    other.reference.referent = Referent::NoRelocation;
    let input_types = [AArch64RelocationType::from_raw(
        object::elf::R_AARCH64_CALL26,
    )];
    assert_eq!(
        comparison_key(&ours.compare(&other), &input_types),
        "rel.missing-opt.R_AARCH64_CALL26.ReplaceWithNop.static-non-pie"
    );
    assert_eq!(
        comparison_key(&other.compare(&ours), &input_types),
        "rel.extra-opt.R_AARCH64_CALL26.ReplaceWithNop.static-non-pie"
    );

    ours.reference.referent = Referent::Named(
        SymbolName {
            bytes: b"target",
            version: None,
        },
        0,
    );
    for comparison in [ours.compare(&other), other.compare(&ours)] {
        assert_eq!(
            comparison_key(&comparison, &input_types),
            "rel.referent-mismatch"
        );
    }
}

#[test]
fn nop_filtering_preserves_equivalence_and_input_indexes() {
    let ours = aarch64_group(&[
        AArch64RelaxationKind::ReplaceWithNop,
        AArch64RelaxationKind::NoOp,
    ]);
    let reference = aarch64_group(&[
        AArch64RelaxationKind::NoOp,
        AArch64RelaxationKind::ReplaceWithNop,
    ]);
    assert!(matches!(
        ours.compare(&reference),
        ResolutionComparison::Equivalent
    ));
    let reference = aarch64_group(&[
        AArch64RelaxationKind::ReplaceWithNop,
        AArch64RelaxationKind::AdrpToAdr,
    ]);
    let input_types = [
        object::elf::R_AARCH64_CALL26,
        object::elf::R_AARCH64_ADR_GOT_PAGE,
    ]
    .map(AArch64RelocationType::from_raw);
    assert_eq!(
        comparison_key(&ours.compare(&reference), &input_types),
        "rel.missing-opt.R_AARCH64_ADR_GOT_PAGE.AdrpToAdr.static-non-pie"
    );
}

#[test]
fn nop_removal_explains_an_unpaired_relocation() {
    let ours = aarch64_group(&[
        AArch64RelaxationKind::NoOp,
        AArch64RelaxationKind::ReplaceWithNop,
    ]);
    let reference = aarch64_group(&[AArch64RelaxationKind::NoOp, AArch64RelaxationKind::NoOp]);
    let input_types = [
        object::elf::R_AARCH64_ADR_GOT_PAGE,
        object::elf::R_AARCH64_CALL26,
    ]
    .map(AArch64RelocationType::from_raw);
    assert_eq!(
        comparison_key(&ours.compare(&reference), &input_types),
        "rel.extra-opt.R_AARCH64_CALL26.ReplaceWithNop.static-non-pie"
    );
    assert_eq!(
        comparison_key(&reference.compare(&ours), &input_types),
        "rel.missing-opt.R_AARCH64_CALL26.ReplaceWithNop.static-non-pie"
    );
}
