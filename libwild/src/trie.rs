//! Builds a Mach-O exports trie.

use crate::verbose_timing_phase;
use leb128::write::unsigned_len as uleb128_size;
use object::macho;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Symbol<'data> {
    pub(crate) name: &'data [u8],
    pub(crate) address: u64,
    pub(crate) flags: macho::ExportSymbolFlags,
}

#[derive(Debug, Default)]
struct Node {
    address: Option<u64>,
    flags: macho::ExportSymbolFlags,
    first_edge: usize,
    num_edges: usize,
    offset: usize,
    size: usize,
}

#[derive(Debug, Default)]
struct Edge<'data> {
    label: &'data [u8],
    child: usize,
    child_offset_size: usize,
}

/// Build a Mach-O exports trie for `symbols`. `symbols` is sorted in place.
pub(crate) fn build(symbols: &mut [Symbol<'_>]) -> Vec<u8> {
    verbose_timing_phase!("Build trie nodes");

    if symbols.is_empty() {
        return Vec::new();
    }

    symbols.sort_unstable_by(|a, b| a.name.cmp(b.name));
    debug_assert!(
        symbols.windows(2).all(|w| w[0].name != w[1].name),
        "duplicate Mach-O export symbol names"
    );

    let mut builder = Builder {
        symbols,
        nodes: Vec::with_capacity(symbols.len() + 1),
        edges: Vec::with_capacity(symbols.len()),
    };
    builder.build_nodes();
    builder.layout_until_stable();
    builder.encode()
}

struct Builder<'data, 'symbols> {
    symbols: &'symbols [Symbol<'data>],
    nodes: Vec<Node>,
    edges: Vec<Edge<'data>>,
}

impl<'data> Builder<'data, '_> {
    fn build_nodes(&mut self) {
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
            end: self.symbols.len(),
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

            let symbol = &self.symbols[start];
            let (address, flags) = if symbol.name.len() == depth {
                start += 1;
                (Some(symbol.address), symbol.flags)
            } else {
                (None, macho::ExportSymbolFlags(0))
            };

            let first_edge = self.edges.len();
            let pending_start = stack.len();
            while start < end {
                let name = self.symbols[start].name;

                // Find the index of the first symbol with a different next byte.
                let child_end = start
                    + 1
                    + self.symbols[start + 1..end]
                        .partition_point(|symbol| symbol.name[depth] == name[depth]);

                let child_depth = if child_end == start + 1 {
                    // There's only a single symbol in this child's range, so the remainder of the
                    // symbol name will be consumed.
                    name.len()
                } else {
                    // The child node will always be at a depth one more than the current node,
                    // but possibly more. e.g. if name="foo" and last_name="fz", then we'll just
                    // bump by 1, but if name="foo" and last_name="fox", we'll bump by 2.
                    let last_name = self.symbols[child_end - 1].name;

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
                    label: &name[depth..child_depth],
                    child: usize::MAX,
                    child_offset_size: 1,
                });

                start = child_end;
            }

            let num_edges = self.edges.len() - first_edge;
            debug_assert!(
                u8::try_from(num_edges).is_ok(),
                "Mach-O exports trie node has too many children"
            );

            self.nodes.push(Node {
                address,
                flags,
                first_edge,
                num_edges,
                ..Default::default()
            });

            // The pending nodes we just added should be visited in the order we added them. Since
            // this is a stack, that means we need to reverse them.
            stack[pending_start..].reverse();
        }
    }

    fn layout_until_stable(&mut self) {
        loop {
            let mut offset = 0;

            for index in 0..self.nodes.len() {
                self.nodes[index].offset = offset;
                self.nodes[index].size = self.node_size(index);
                offset += self.nodes[index].size;
            }

            let mut changed = false;

            for edge in &mut self.edges {
                let offset_size = uleb128_size(self.nodes[edge.child].offset as u64);
                if edge.child_offset_size != offset_size {
                    edge.child_offset_size = offset_size;
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }
    }

    fn node_size(&self, node_index: usize) -> usize {
        let node = &self.nodes[node_index];
        let terminal_size = node
            .address
            .map_or(0, |address| regular_export_size(node.flags, address));
        uleb128_size(terminal_size as u64)
            + terminal_size
            + 1
            + self
                .node_edges(node_index)
                .map(|edge| edge.label.len() + 1 + edge.child_offset_size)
                .sum::<usize>()
    }

    fn encode(&self) -> Vec<u8> {
        let total_size = self.nodes.last().map_or(0, |node| node.offset + node.size);
        let mut out = Vec::with_capacity(total_size);

        for (node_index, node) in self.nodes.iter().enumerate() {
            debug_assert_eq!(out.len(), node.offset);

            if let Some(address) = node.address {
                write_uleb128(&mut out, regular_export_size(node.flags, address) as u64);
                write_regular_export(&mut out, node.flags, address);
            } else {
                write_uleb128(&mut out, 0);
            }

            out.push(node.num_edges as u8);

            for edge in self.node_edges(node_index) {
                out.extend_from_slice(edge.label);
                out.push(0);
                write_uleb128(&mut out, self.nodes[edge.child].offset as u64);
            }
        }

        debug_assert_eq!(out.len(), total_size);
        out
    }

    fn node_edges(&self, node_index: usize) -> impl Iterator<Item = &Edge<'data>> {
        let node = &self.nodes[node_index];
        self.edges[node.first_edge..node.first_edge + node.num_edges].iter()
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
        let trie = build(symbols);

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
        let mut symbols = [];
        assert!(build(&mut symbols).is_empty());
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
            .map(|index| format!("_shared_prefix_{index:04x}").into_bytes())
            .collect_vec();

        let mut actual = names
            .iter()
            .enumerate()
            .map(|(index, name)| Symbol {
                name,
                address: 1_u64 << (index % 63),
                flags: macho::ExportSymbolFlags(0),
            })
            .collect_vec();

        let mut maximum = names
            .iter()
            .map(|name| Symbol {
                name,
                address: u64::MAX,
                flags: macho::ExportSymbolFlags(0),
            })
            .collect_vec();

        assert!(build(&mut actual).len() <= build(&mut maximum).len());
    }
}
