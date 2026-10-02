//! Builds a Mach-O exports trie.

use crate::verbose_timing_phase;
use leb128::write::unsigned_len as uleb128_size;
use object::macho;
use std::ops::Range;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Symbol<'data> {
    pub(crate) name: &'data [u8],
    pub(crate) address: u64,
    pub(crate) flags: macho::ExportSymbolFlags,
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
    pub(crate) fn new(symbols: &[Symbol]) -> Self {
        verbose_timing_phase!("Prepare exports trie");

        let mut names = symbols
            .iter()
            .enumerate()
            .map(|(index, symbol)| (index, symbol.name))
            .collect::<Vec<_>>();
        {
            verbose_timing_phase!("Sort trie symbols");
            names.sort_unstable_by(|a, b| a.1.cmp(b.1));
        }

        debug_assert!(
            names.windows(2).all(|w| w[0].1 != w[1].1),
            "duplicate Mach-O export symbol names"
        );

        let mut trie = Self {
            nodes: Vec::with_capacity(symbols.len() + 1),
            edges: Vec::with_capacity(symbols.len()),
            num_symbols: symbols.len(),
        };

        if !symbols.is_empty() {
            trie.build_nodes(&names);
        }

        trie
    }

    /// Compute the size that would be returned by `build`, but without doing so much work. Symbol
    /// names and their order must match the input to `new`. Addresses and flags may be different.
    pub(crate) fn compute_byte_size(&self, symbols: &[Symbol]) -> usize {
        verbose_timing_phase!("Size exports trie");
        Layout::new(self, symbols).encoded_size()
    }

    /// Encode the trie as bytes. Symbol names and their order must match the input to `new`.
    pub(crate) fn encode(&self, symbols: &[Symbol]) -> Vec<u8> {
        verbose_timing_phase!("Encode exports trie");
        Layout::new(self, symbols).encode()
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

struct Layout<'trie, 'symbols, 'data> {
    trie: &'trie Trie,
    symbols: &'symbols [Symbol<'data>],
    node_offsets: Vec<usize>,
    child_offset_sizes: Vec<usize>,
    total_size: usize,
}

impl<'trie, 'symbols, 'data> Layout<'trie, 'symbols, 'data> {
    fn new(trie: &'trie Trie, symbols: &'symbols [Symbol<'data>]) -> Self {
        debug_assert_eq!(trie.num_symbols, symbols.len());

        let mut layout = Self {
            trie,
            symbols,
            node_offsets: vec![0; trie.nodes.len()],
            child_offset_sizes: vec![1; trie.edges.len()],
            total_size: 0,
        };

        layout.layout_until_stable();

        layout
    }

    fn layout_until_stable(&mut self) {
        verbose_timing_phase!("Layout trie");

        loop {
            let mut offset = 0;
            for index in 0..self.node_offsets.len() {
                self.node_offsets[index] = offset;
                offset += self.node_size(index);
            }

            let mut changed = false;
            for (edge, size) in self.trie.edges.iter().zip(&mut self.child_offset_sizes) {
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

    fn node_size(&self, node_index: usize) -> usize {
        let node = &self.trie.nodes[node_index];
        let terminal_size = node.symbol.map_or(0, |index| {
            let symbol = &self.symbols[index];
            regular_export_size(symbol.flags, symbol.address)
        });

        uleb128_size(terminal_size as u64)
            + terminal_size
            + 1
            + self
                .trie
                .node_edges(node_index)
                .zip(&self.child_offset_sizes[node.first_edge..])
                .map(|(edge, offset_size)| edge.label.len() + 1 + offset_size)
                .sum::<usize>()
    }

    fn encoded_size(&self) -> usize {
        self.total_size
    }

    fn encode(&self) -> Vec<u8> {
        verbose_timing_phase!("Encode trie");

        let total_size = self.encoded_size();
        let mut out = Vec::with_capacity(total_size);

        for (node_index, node) in self.trie.nodes.iter().enumerate() {
            debug_assert_eq!(out.len(), self.node_offsets[node_index]);

            if let Some(index) = node.symbol {
                let symbol = &self.symbols[index];

                write_uleb128(
                    &mut out,
                    regular_export_size(symbol.flags, symbol.address) as u64,
                );

                write_regular_export(&mut out, symbol.flags, symbol.address);
            } else {
                write_uleb128(&mut out, 0);
            }

            out.push(node.num_edges as u8);

            for edge in self.trie.node_edges(node_index) {
                out.extend_from_slice(&self.symbols[edge.symbol].name[edge.label.clone()]);
                out.push(0);
                write_uleb128(&mut out, self.node_offsets[edge.child] as u64);
            }
        }

        debug_assert_eq!(out.len(), total_size);
        out
    }
}

fn regular_export_size(flags: macho::ExportSymbolFlags, address: u64) -> usize {
    uleb128_size(flags.0) + uleb128_size(address)
}

fn write_regular_export(out: &mut Vec<u8>, flags: macho::ExportSymbolFlags, address: u64) {
    write_uleb128(out, flags.0);
    write_uleb128(out, address);
}

fn write_uleb128(out: &mut Vec<u8>, value: u64) {
    leb128::write::unsigned(out, value).unwrap();
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
        address: u64,
        flags: macho::ExportSymbolFlags,
    }

    fn check(symbols: &mut [Symbol]) {
        let trie = Trie::new(symbols);
        let size = trie.compute_byte_size(symbols);
        let trie = trie.encode(symbols);
        assert_eq!(size, trie.len());
        symbols.sort_unstable_by(|a, b| a.name.cmp(b.name));

        assert_eq!(
            parse_exports(&trie),
            symbols
                .iter()
                .map(|s| ParsedSymbol {
                    name: s.name.to_owned(),
                    address: s.address,
                    flags: s.flags,
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
                let ExportData::Regular { address } = symbol.data() else {
                    panic!("expected regular export");
                };
                ParsedSymbol {
                    name: symbol.name().to_vec(),
                    address: *address,
                    flags: symbol.flags(),
                }
            })
            .collect()
    }

    #[test]
    fn empty_input_produces_empty_trie() {
        let symbols = [];
        let trie = Trie::new(&symbols);
        assert_eq!(trie.compute_byte_size(&symbols), 0);
        assert!(trie.encode(&symbols).is_empty());
    }

    #[test]
    fn builds_single_symbol_trie() {
        check(&mut [Symbol {
            name: b"_main",
            address: 0x1234,
            flags: macho::ExportSymbolFlags(0),
        }]);
    }

    #[test]
    fn builds_absolute_symbol() {
        let mut symbols = [Symbol {
            name: b"_absolute",
            address: 42,
            flags: macho::EXPORT_SYMBOL_FLAGS_KIND_ABSOLUTE.into(),
        }];

        check(&mut symbols);
    }

    #[test]
    fn builds_weak_symbol() {
        let mut symbols = [Symbol {
            name: b"_weak",
            address: 42,
            flags: macho::EXPORT_SYMBOL_FLAGS_WEAK_DEFINITION,
        }];

        check(&mut symbols);
    }

    #[test]
    fn builds_shared_prefix_trie() {
        check(&mut [
            Symbol {
                name: b"_foobar",
                address: 1,
                flags: macho::ExportSymbolFlags(0),
            },
            Symbol {
                name: b"_foo",
                address: 2,
                flags: macho::ExportSymbolFlags(0),
            },
            Symbol {
                name: b"_fop",
                address: 3,
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
                address: address as u64,
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
                address: index as u64,
                flags: macho::ExportSymbolFlags(0),
            })
            .collect();

        check(&mut symbols);
    }

    #[test]
    fn maximum_addresses_give_a_conservative_size() {
        let names = (0..512)
            .rev()
            .map(|index| format!("_shared_prefix_{index:04x}").into_bytes())
            .collect_vec();

        let actual = names
            .iter()
            .enumerate()
            .map(|(index, name)| Symbol {
                name,
                address: 1_u64 << (index % 63),
                flags: if index % 2 == 0 {
                    macho::EXPORT_SYMBOL_FLAGS_WEAK_DEFINITION
                } else {
                    macho::EXPORT_SYMBOL_FLAGS_KIND_ABSOLUTE.into()
                },
            })
            .collect_vec();

        let maximum = names
            .iter()
            .map(|name| Symbol {
                name,
                address: u64::MAX,
                flags: macho::ExportSymbolFlags(0),
            })
            .collect_vec();

        let trie = Trie::new(&maximum);
        let maximum_size = trie.compute_byte_size(&maximum);
        let encoded = trie.encode(&actual);
        assert!(encoded.len() <= maximum_size);
        assert_eq!(encoded.len(), trie.compute_byte_size(&actual));
        assert_eq!(encoded, Trie::new(&actual).encode(&actual));
        assert_eq!(
            parse_exports(&encoded),
            actual
                .iter()
                .sorted_unstable_by_key(|symbol| symbol.name)
                .map(|symbol| ParsedSymbol {
                    name: symbol.name.to_vec(),
                    address: symbol.address,
                    flags: symbol.flags,
                })
                .collect_vec()
        );
    }
}
