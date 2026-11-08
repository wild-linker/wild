//! Builds a Mach-O exports trie.

use crate::ensure;
use crate::error::Result;
use crate::verbose_timing_phase;
use leb128::write::unsigned_len as uleb128_size;
use object::macho;
use rayon::slice::ParallelSliceMut;
use std::io::Write;
use std::ops::Range;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    Regular { address: u64 },
    Reexport { dylib_ordinal: u64 },
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Terminal {
    pub(crate) target: Target,
    pub(crate) flags: macho::ExportSymbolFlags,
}

impl Terminal {
    fn export_flags(&self) -> macho::ExportSymbolFlags {
        match self.target {
            Target::Regular { .. } => self.flags,
            Target::Reexport { .. } => self.flags | macho::EXPORT_SYMBOL_FLAGS_REEXPORT,
        }
    }

    fn encoded_size(&self) -> usize {
        uleb128_size(self.export_flags().0)
            + match self.target {
                Target::Regular { address } => uleb128_size(address),
                Target::Reexport { dylib_ordinal } => uleb128_size(dylib_ordinal) + 1,
            }
    }

    fn write(&self, out: &mut &mut [u8]) -> Result {
        write_uleb128(out, self.export_flags().0)?;
        match self.target {
            Target::Regular { address } => write_uleb128(out, address)?,
            Target::Reexport { dylib_ordinal } => {
                write_uleb128(out, dylib_ordinal)?;
                // An empty import name forwards the exported name unchanged.
                out.write_all(&[0])?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Node {
    symbol: Option<usize>,
    first_edge: usize,
    num_edges: usize,
}

#[derive(Debug)]
struct Edge {
    symbol: usize,
    label: Range<usize>,
    child: usize,
}

/// The structure of an exports trie, independent of symbol addresses and flags. Symbol indexes
/// refer to the original input order, which must also be used when sizing and encoding the trie.
#[derive(Debug, Default)]
pub(crate) struct Trie {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    num_symbols: usize,
}

impl Trie {
    pub(crate) fn new<'data>(names: impl IntoIterator<Item = &'data [u8]>) -> Self {
        verbose_timing_phase!("Prepare exports trie");

        let mut names = names.into_iter().enumerate().collect::<Vec<_>>();
        {
            verbose_timing_phase!("Sort trie symbols");
            names.par_sort_unstable_by_key(|&(_, name)| name);
        }

        debug_assert!(
            names.windows(2).all(|w| w[0].1 != w[1].1),
            "duplicate Mach-O export symbol names"
        );

        let mut trie = Self {
            nodes: Vec::with_capacity(names.len() + 1),
            edges: Vec::with_capacity(names.len()),
            num_symbols: names.len(),
        };

        if !names.is_empty() {
            trie.build_nodes(&names);
        }

        trie
    }

    /// Prepare exact node offsets without encoding bytes. Terminals must follow the name order
    /// passed to `new`. The prepared trie retains the topology and final values for later encoding.
    pub(crate) fn prepare(self: &Arc<Self>, terminals: Vec<Terminal>) -> PreparedTrie {
        verbose_timing_phase!("Size exports trie");
        PreparedTrie::new(self, terminals)
    }

    fn build_nodes(&mut self, symbols: &[(usize, &[u8])]) {
        verbose_timing_phase!("Build trie nodes");

        struct PendingNode {
            /// The range of offsets into `symbols` that the node represents.
            start: usize,
            end: usize,

            /// How many bytes into the symbol name we are.
            depth: usize,

            parent_edge: Option<usize>,
        }

        let mut stack = vec![PendingNode {
            start: 0,
            end: symbols.len(),
            depth: 0,
            parent_edge: None,
        }];

        while let Some(PendingNode {
            mut start,
            end,
            depth,
            parent_edge,
        }) = stack.pop()
        {
            let node_index = self.nodes.len();
            if let Some(edge_index) = parent_edge {
                self.edges[edge_index].child = node_index;
            }

            let symbol = if symbols[start].1.len() == depth {
                let symbol = Some(symbols[start].0);
                start += 1;
                symbol
            } else {
                None
            };

            let first_edge = self.edges.len();
            let pending_start = stack.len();
            while start < end {
                let (symbol_index, name) = symbols[start];

                // Find the index of the first symbol with a different next byte.
                let child_end = start
                    + 1
                    + symbols[start + 1..end]
                        .partition_point(|symbol| symbol.1[depth] == name[depth]);

                let child_depth = if child_end == start + 1 {
                    // There's only a single symbol in this child's range, so the remainder of the
                    // symbol name will be consumed.
                    name.len()
                } else {
                    // The child node will always be at a depth one more than the current node,
                    // but possibly more. e.g. if name="foo" and last_name="fz", then we'll just
                    // bump by 1, but if name="foo" and last_name="fox", we'll bump by 2.
                    let last_name = symbols[child_end - 1].1;

                    depth
                        + 1
                        + name[depth + 1..]
                            .iter()
                            .zip(&last_name[depth + 1..])
                            .take_while(|(a, b)| a == b)
                            .count()
                };

                stack.push(PendingNode {
                    start,
                    end: child_end,
                    depth: child_depth,
                    parent_edge: Some(self.edges.len()),
                });

                self.edges.push(Edge {
                    symbol: symbol_index,
                    label: depth..child_depth,
                    child: usize::MAX,
                });

                start = child_end;
            }

            let num_edges = self.edges.len() - first_edge;
            debug_assert!(
                u8::try_from(num_edges).is_ok(),
                "Mach-O exports trie node has too many children"
            );

            self.nodes.push(Node {
                symbol,
                first_edge,
                num_edges,
            });

            // The pending nodes we just added should be visited in the order we added them. Since
            // this is a stack, that means we need to reverse them.
            stack[pending_start..].reverse();
        }
    }

    fn node_edges(&self, node_index: usize) -> impl Iterator<Item = &Edge> {
        let node = &self.nodes[node_index];
        self.edges[node.first_edge..node.first_edge + node.num_edges].iter()
    }
}

/// Retains the topology, exact node offsets and final symbol values for later encoding without
/// repeating symbol resolution or layout.
#[derive(Debug, Default)]
pub(crate) struct PreparedTrie {
    trie: Arc<Trie>,
    node_offsets: Vec<usize>,
    terminals: Vec<Terminal>,
    total_size: usize,
}

impl PreparedTrie {
    fn new(trie: &Arc<Trie>, terminals: Vec<Terminal>) -> Self {
        debug_assert_eq!(trie.num_symbols, terminals.len());

        let mut layout = Self {
            trie: Arc::clone(trie),
            node_offsets: vec![0; trie.nodes.len()],
            terminals,
            total_size: 0,
        };
        let mut child_offset_sizes = vec![1; trie.edges.len()];
        layout.layout_until_stable(trie, &mut child_offset_sizes);
        layout
    }

    fn layout_until_stable(&mut self, trie: &Trie, child_offset_sizes: &mut [usize]) {
        verbose_timing_phase!("Layout trie");

        loop {
            let mut offset = 0;
            for index in 0..self.node_offsets.len() {
                self.node_offsets[index] = offset;
                offset += self.node_size(trie, index, child_offset_sizes);
            }

            let mut changed = false;
            for (edge, size) in trie.edges.iter().zip(child_offset_sizes.iter_mut()) {
                let offset_size = uleb128_size(self.node_offsets[edge.child] as u64);
                if *size != offset_size {
                    *size = offset_size;
                    changed = true;
                }
            }

            if !changed {
                self.total_size = offset;
                break;
            }
        }
    }

    fn node_size(&self, trie: &Trie, node_index: usize, child_offset_sizes: &[usize]) -> usize {
        let node = &trie.nodes[node_index];
        let terminal_size = node.symbol.map_or(0, |index| {
            let symbol = &self.terminals[index];
            symbol.encoded_size()
        });

        uleb128_size(terminal_size as u64)
            + terminal_size
            + 1
            + trie
                .node_edges(node_index)
                .zip(&child_offset_sizes[node.first_edge..])
                .map(|(edge, offset_size)| edge.label.len() + 1 + offset_size)
                .sum::<usize>()
    }

    pub(crate) fn encoded_size(&self) -> usize {
        self.total_size
    }

    /// Encode directly into an output buffer of exactly `encoded_size()` bytes using the retained
    /// topology, offsets and symbol values. The caller must supply the same names, in the same
    /// order, as those passed to `Trie::new`.
    pub(crate) fn encode_into<'data>(
        &self,
        mut out: &mut [u8],
        symbol_name: impl Fn(usize) -> &'data [u8],
    ) -> Result {
        verbose_timing_phase!("Encode exports trie");
        let trie = &self.trie;
        debug_assert_eq!(trie.num_symbols, self.terminals.len());
        debug_assert_eq!(trie.nodes.len(), self.node_offsets.len());

        let total_size = self.encoded_size();
        ensure!(
            out.len() == total_size,
            "Mach-O exports trie size differs from its allocation"
        );

        for (node_index, node) in trie.nodes.iter().enumerate() {
            debug_assert_eq!(total_size - out.len(), self.node_offsets[node_index]);

            if let Some(index) = node.symbol {
                let symbol = &self.terminals[index];

                write_uleb128(&mut out, symbol.encoded_size() as u64)?;

                symbol.write(&mut out)?;
            } else {
                write_uleb128(&mut out, 0)?;
            }

            out.write_all(&[node.num_edges as u8])?;

            for edge in trie.node_edges(node_index) {
                out.write_all(&symbol_name(edge.symbol)[edge.label.clone()])?;
                out.write_all(&[0])?;
                write_uleb128(&mut out, self.node_offsets[edge.child] as u64)?;
            }
        }

        ensure!(out.is_empty(), "Excess Mach-O exports trie allocation");
        Ok(())
    }
}

fn write_uleb128(out: &mut &mut [u8], value: u64) -> Result {
    leb128::write::unsigned(out, value)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use object::LittleEndian;
    use object::macho;
    use object::read::macho::ExportData;

    #[derive(Debug, PartialEq, Eq)]
    struct ParsedSymbol {
        name: Vec<u8>,
        target: Target,
        flags: macho::ExportSymbolFlags,
    }

    #[derive(Debug, Clone, Copy)]
    struct Symbol<'data> {
        name: &'data [u8],
        target: Target,
        flags: macho::ExportSymbolFlags,
    }

    impl Symbol<'_> {
        fn terminal(&self) -> Terminal {
            Terminal {
                target: self.target,
                flags: self.flags,
            }
        }
    }

    fn check(symbols: &mut [Symbol]) {
        let trie = Arc::new(Trie::new(symbols.iter().map(|symbol| symbol.name)));
        let layout = trie.prepare(symbols.iter().map(Symbol::terminal).collect());
        // The output buffer need not be zeroed, including bytes used as terminators.
        let mut trie = vec![0xa5; layout.encoded_size()];
        layout
            .encode_into(&mut trie, |index| symbols[index].name)
            .unwrap();
        symbols.sort_unstable_by(|a, b| a.name.cmp(b.name));

        assert_eq!(
            parse_exports(&trie),
            symbols
                .iter()
                .map(|s| ParsedSymbol {
                    name: s.name.to_owned(),
                    target: s.target,
                    flags: s.terminal().export_flags(),
                })
                .collect_vec()
        );
    }

    fn parse_exports(data: &[u8]) -> Vec<ParsedSymbol> {
        if data.is_empty() {
            return Vec::new();
        }

        let command = macho::LinkeditDataCommand {
            cmd: macho::LC_DYLD_EXPORTS_TRIE.into(),
            cmdsize: (size_of::<macho::LinkeditDataCommand<object::Endianness>>() as u32).into(),
            dataoff: 0.into(),
            datasize: (data.len() as u32).into(),
        };

        command
            .exports_trie(LittleEndian, data)
            .unwrap()
            .map(|symbol| {
                let symbol = symbol.unwrap();
                let target = match symbol.data() {
                    ExportData::Regular { address } => Target::Regular { address: *address },
                    ExportData::Reexport {
                        dylib_ordinal,
                        import_name,
                    } => {
                        assert!(import_name.is_empty());
                        Target::Reexport {
                            dylib_ordinal: *dylib_ordinal,
                        }
                    }
                    ExportData::StubAndResolver { .. } => panic!("unexpected stub and resolver"),
                };
                ParsedSymbol {
                    name: symbol.name().to_vec(),
                    target,
                    flags: symbol.flags(),
                }
            })
            .collect()
    }

    #[test]
    fn reexports_round_trip() {
        check(&mut [
            Symbol {
                name: b"_foo",
                flags: macho::ExportSymbolFlags(0),
                target: Target::Regular { address: 42 },
            },
            Symbol {
                name: b"_foobar",
                flags: macho::ExportSymbolFlags(0),
                target: Target::Reexport { dylib_ordinal: 128 },
            },
            Symbol {
                name: b"_bar",
                flags: macho::ExportSymbolFlags(0),
                target: Target::Reexport { dylib_ordinal: 2 },
            },
        ]);
    }

    #[test]
    fn empty_input_produces_empty_trie() {
        let trie = Arc::new(Trie::new([]));
        let layout = trie.prepare(Vec::new());
        assert_eq!(layout.encoded_size(), 0);
        layout.encode_into(&mut [], |_| unreachable!()).unwrap();
    }

    #[test]
    fn builds_single_symbol_trie() {
        check(&mut [Symbol {
            name: b"_main",
            target: Target::Regular { address: 0x1234 },
            flags: macho::ExportSymbolFlags(0),
        }]);
    }

    #[test]
    fn builds_absolute_symbol() {
        let mut symbols = [Symbol {
            name: b"_absolute",
            target: Target::Regular { address: 42 },
            flags: macho::EXPORT_SYMBOL_FLAGS_KIND_ABSOLUTE.into(),
        }];

        check(&mut symbols);
    }

    #[test]
    fn builds_weak_symbol() {
        let mut symbols = [Symbol {
            name: b"_weak",
            target: Target::Regular { address: 42 },
            flags: macho::EXPORT_SYMBOL_FLAGS_WEAK_DEFINITION,
        }];

        check(&mut symbols);
    }

    #[test]
    fn builds_thread_local_symbol() {
        let mut symbols = [Symbol {
            name: b"_tls",
            target: Target::Regular { address: 42 },
            flags: macho::EXPORT_SYMBOL_FLAGS_KIND_THREAD_LOCAL.into(),
        }];

        check(&mut symbols);
    }

    #[test]
    fn builds_shared_prefix_trie() {
        check(&mut [
            Symbol {
                name: b"_foobar",
                target: Target::Regular { address: 1 },
                flags: macho::ExportSymbolFlags(0),
            },
            Symbol {
                name: b"_foo",
                target: Target::Regular { address: 2 },
                flags: macho::ExportSymbolFlags(0),
            },
            Symbol {
                name: b"_fop",
                target: Target::Regular { address: 3 },
                flags: macho::ExportSymbolFlags(0),
            },
        ]);
    }

    #[test]
    fn builds_deeply_nested_prefixes() {
        let names = (1..=1024).map(|len| vec![b'a'; len]).collect_vec();
        let mut symbols = names
            .iter()
            .enumerate()
            .map(|(address, name)| Symbol {
                name,
                target: Target::Regular {
                    address: address as u64,
                },
                flags: macho::ExportSymbolFlags(0),
            })
            .collect_vec();

        check(&mut symbols);
    }

    #[test]
    fn every_non_zero_byte() {
        let names: Vec<Vec<u8>> = (1..=255).map(|n| vec![n]).collect();
        let mut symbols: Vec<_> = names
            .iter()
            .enumerate()
            .map(|(index, name)| Symbol {
                name,
                target: Target::Regular {
                    address: index as u64,
                },
                flags: macho::ExportSymbolFlags(0),
            })
            .collect();

        check(&mut symbols);
    }

    #[test]
    fn topology_reused_with_different_terminals() {
        // Deliberately unsorted, with both a shared prefix and a terminal on an internal node.
        let names: [&[u8]; 3] = [b"_foobar", b"_bar", b"_foo"];
        let trie = Arc::new(Trie::new(names));

        for (addresses, flags) in [
            ([127, 0, 16384], macho::ExportSymbolFlags(0)),
            (
                [128, u64::MAX, 42],
                macho::EXPORT_SYMBOL_FLAGS_WEAK_DEFINITION,
            ),
        ] {
            let terminals = addresses
                .iter()
                .map(|&address| Terminal {
                    target: Target::Regular { address },
                    flags,
                })
                .collect();
            let prepared = trie.prepare(terminals);
            let mut encoded = vec![0xa5; prepared.encoded_size()];
            prepared
                .encode_into(&mut encoded, |index| names[index])
                .unwrap();
            assert_eq!(
                parse_exports(&encoded),
                names
                    .into_iter()
                    .zip(addresses)
                    .sorted_unstable_by_key(|&(name, _)| name)
                    .map(|(name, address)| ParsedSymbol {
                        name: name.to_vec(),
                        target: Target::Regular { address },
                        flags,
                    })
                    .collect_vec()
            );
        }
    }

    #[test]
    fn uleb128_boundaries() {
        // The root initially occupies name.len() + 4 bytes. At a ULEB128 boundary, growing the
        // child offset also moves the child, so its final offset must use the new root size.
        for boundary in [128, 16384] {
            for length in boundary - 5..=boundary - 3 {
                let name = vec![b'a'; length];
                for address in [127, 128, 16383, 16384, u64::MAX] {
                    check(&mut [Symbol {
                        name: &name,
                        target: Target::Regular { address },
                        flags: macho::ExportSymbolFlags(0),
                    }]);
                }
            }
        }
    }
}
