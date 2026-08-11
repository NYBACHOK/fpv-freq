use std::collections::HashSet;

pub struct FpvConflictResolver {
    pub min_separation: i32,
}

impl FpvConflictResolver {
    pub fn new(min_separation: i32) -> Self {
        Self { min_separation }
    }

    /// Evaluates all channels against active channels and returns all incompatible channel IDs.
    pub fn find_incompatible_ids(
        &self,
        all_channels: &[(String, i32)],
        active_channels: &[(String, i32)],
    ) -> HashSet<String> {
        let mut incompatible = HashSet::new();

        if active_channels.is_empty() {
            return incompatible;
        }

        for (candidate_id, candidate_freq) in all_channels {
            for (active_id, active_freq) in active_channels {
                if candidate_id == active_id {
                    continue;
                }

                // 1. Direct Frequency Overlap (<= min_separation)
                if (candidate_freq - active_freq).abs() <= self.min_separation {
                    incompatible.insert(candidate_id.clone());
                }
            }

            // 2. 3rd-Order Intermodulation (IMD3 = 2 * fA - fB)
            for (id_a, freq_a) in active_channels {
                for (id_b, freq_b) in active_channels {
                    if id_a == id_b {
                        continue;
                    }

                    let imd3_freq = (2 * freq_a) - freq_b;
                    if (candidate_freq - imd3_freq).abs() <= self.min_separation {
                        incompatible.insert(candidate_id.clone());
                    }
                }
            }
        }

        incompatible
    }
}
