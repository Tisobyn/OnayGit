//! Commit graph: assign every commit a lane and describe the lines drawn in its row.
use super::*;

#[derive(Clone, Debug, Default)]
pub struct GraphRow {
    /// Lane of the commit dot.
    pub col: usize,
    /// Lanes that continue from the row above.
    pub top: Vec<bool>,
    /// Lanes that continue into the row below.
    pub bottom: Vec<bool>,
    /// Lanes from above that end in this commit (merge).
    pub merge_from: Vec<usize>,
    /// Lanes below that start from this commit (extra parents).
    pub branch_to: Vec<usize>,
}

/// Returns one row per commit and the number of lanes used.
pub fn graph(commits: &[Commit]) -> (Vec<GraphRow>, usize) {
    let mut lanes: Vec<Option<String>> = Vec::new();
    let mut rows = Vec::with_capacity(commits.len());
    let mut width = 1;

    for c in commits {
        let top: Vec<bool> = lanes.iter().map(|l| l.is_some()).collect();
        let matching: Vec<usize> = lanes
            .iter()
            .enumerate()
            .filter(|(_, l)| l.as_deref() == Some(c.hash.as_str()))
            .map(|(i, _)| i)
            .collect();

        let col = match matching.first() {
            Some(&i) => i,
            None => match lanes.iter().position(|l| l.is_none()) {
                Some(i) => i,
                None => {
                    lanes.push(None);
                    lanes.len() - 1
                }
            },
        };
        let merge_from: Vec<usize> = matching.iter().skip(1).copied().collect();
        for &i in &merge_from {
            lanes[i] = None;
        }

        let mut branch_to = Vec::new();
        match c.parents.split_first() {
            None => lanes[col] = None,
            Some((first, rest)) => {
                lanes[col] = Some(first.clone());
                for p in rest {
                    let lane = match lanes.iter().position(|l| l.as_deref() == Some(p.as_str())) {
                        Some(i) => i,
                        None => {
                            let i = match lanes.iter().position(|l| l.is_none()) {
                                Some(i) => i,
                                None => {
                                    lanes.push(None);
                                    lanes.len() - 1
                                }
                            };
                            lanes[i] = Some(p.clone());
                            i
                        }
                    };
                    branch_to.push(lane);
                }
            }
        }

        let bottom: Vec<bool> = lanes.iter().map(|l| l.is_some()).collect();
        width = width.max(lanes.len());
        rows.push(GraphRow { col, top, bottom, merge_from, branch_to });
    }
    (rows, width)
}

/// Branches and tags with the commit each one points at: (name, kind 1 local / 2 remote / 3 tag, hash).
pub fn ref_list(repo: &Path) -> Vec<(String, u8, String)> {
    run(repo, &["for-each-ref", "--format=%(refname)%09%(objectname)%09%(*objectname)"])
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut p = l.split('\t');
            let full = p.next()?;
            let obj = p.next()?;
            let peeled = p.next().unwrap_or("");
            let hash = if peeled.is_empty() { obj } else { peeled }.to_string();
            let (kind, name) = if let Some(n) = full.strip_prefix("refs/heads/") {
                (1, n)
            } else if let Some(n) = full.strip_prefix("refs/remotes/") {
                if n.ends_with("/HEAD") {
                    return None;
                }
                (2, n)
            } else if let Some(n) = full.strip_prefix("refs/tags/") {
                (3, n)
            } else {
                return None;
            };
            Some((name.to_string(), kind, hash))
        })
        .collect()
}
