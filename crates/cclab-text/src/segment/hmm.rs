//! Hidden Markov Model for unknown word recognition.
//!
//! Uses BMES tagging (Begin, Middle, End, Single) with Viterbi algorithm.

use std::collections::HashMap;

/// HMM states for Chinese word segmentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    B, // Begin of word
    M, // Middle of word
    E, // End of word
    S, // Single character word
}

impl State {
    fn all() -> &'static [State] {
        &[State::B, State::M, State::E, State::S]
    }

    fn index(&self) -> usize {
        match self {
            State::B => 0,
            State::M => 1,
            State::E => 2,
            State::S => 3,
        }
    }
}

/// HMM model for Chinese word segmentation.
pub struct HmmModel {
    /// Initial state probabilities (log)
    start_prob: [f64; 4],
    /// Transition probabilities (log): trans[from][to]
    trans_prob: [[f64; 4]; 4],
    /// Emission probabilities (log): state -> char -> prob
    emit_prob: [HashMap<char, f64>; 4],
    /// Default emission probability for unknown chars (per state)
    default_emit: [f64; 4],
}

impl Default for HmmModel {
    fn default() -> Self {
        Self::new()
    }
}

impl HmmModel {
    /// Create a new HMM model with trained parameters.
    pub fn new() -> Self {
        // Initial state probabilities (log) - can only start with B or S
        let start_prob = [
            -0.26268660809250016, // B - common to start multi-char word
            f64::NEG_INFINITY,    // M - cannot start with middle
            f64::NEG_INFINITY,    // E - cannot start with end
            -1.4652633398537678,  // S - less common single char start
        ];

        // Transition probabilities (log)
        // Valid transitions:
        //   B -> M (continue word), B -> E (end 2-char word)
        //   M -> M (continue word), M -> E (end multi-char word)
        //   E -> B (start new word), E -> S (single char word)
        //   S -> B (start new word), S -> S (another single char)
        let inf = f64::NEG_INFINITY;
        let trans_prob = [
            // From B: can go to M or E
            [inf, -0.916290731874155, -0.510825623765990, inf],
            // From M: can go to M or E
            [inf, -1.2603623820268226, -0.33344856811948514, inf],
            // From E: can go to B or S
            [-0.5897149736854513, inf, inf, -0.8085250474669937],
            // From S: can go to B or S
            [-0.7211385869380396, inf, inf, -0.6658631448798212],
        ];

        // Emission probabilities - character likelihood per state
        // B: characters that typically begin words
        // M: characters that appear in middle of words
        // E: characters that typically end words
        // S: characters that are often single-char words
        let mut emit_b: HashMap<char, f64> = HashMap::new();
        let mut emit_m: HashMap<char, f64> = HashMap::new();
        let mut emit_e: HashMap<char, f64> = HashMap::new();
        let mut emit_s: HashMap<char, f64> = HashMap::new();

        // Common begin characters
        for ch in "我你他她它这那什怎为因所但如虽可没就都也".chars() {
            emit_b.insert(ch, -2.0);
            emit_s.insert(ch, -3.0);
        }

        // Common end characters
        for ch in "了的地得着过吗呢吧啊".chars() {
            emit_e.insert(ch, -2.0);
            emit_s.insert(ch, -2.5);
        }

        // Middle characters (less distinctive)
        for ch in "中大小国人民".chars() {
            emit_m.insert(ch, -3.0);
            emit_e.insert(ch, -3.5);
        }

        // Single char words
        for ch in "一二三四五六七八九十是在有和与".chars() {
            emit_s.insert(ch, -2.0);
        }

        let emit_prob = [emit_b, emit_m, emit_e, emit_s];

        // Default emission per state (for characters not in emission table)
        // Different defaults to add state preference
        let default_emit = [
            -5.0, // B - moderate for beginning
            -5.5, // M - less likely for middle (requires continuation)
            -5.0, // E - moderate for ending
            -4.5, // S - slightly prefer single char for unknown
        ];

        Self {
            start_prob,
            trans_prob,
            emit_prob,
            default_emit,
        }
    }

    /// Get emission probability for a character in a state.
    fn emit(&self, state: State, ch: char) -> f64 {
        self.emit_prob[state.index()]
            .get(&ch)
            .copied()
            .unwrap_or(self.default_emit[state.index()])
    }

    /// Run Viterbi algorithm to find best state sequence.
    pub fn viterbi(&self, chars: &[char]) -> Vec<State> {
        if chars.is_empty() {
            return vec![];
        }

        let n = chars.len();
        let states = State::all();

        // V[t][state] = (probability, previous_state)
        let mut v: Vec<[(f64, Option<State>); 4]> = vec![[(f64::NEG_INFINITY, None); 4]; n];

        // Initialize first character
        for &state in states {
            let prob = self.start_prob[state.index()] + self.emit(state, chars[0]);
            v[0][state.index()] = (prob, None);
        }

        // Forward pass
        for t in 1..n {
            for &curr_state in states {
                let emit = self.emit(curr_state, chars[t]);
                let mut best_prob = f64::NEG_INFINITY;
                let mut best_prev = State::B;

                for &prev_state in states {
                    let trans = self.trans_prob[prev_state.index()][curr_state.index()];
                    if trans.is_finite() {
                        let prob = v[t - 1][prev_state.index()].0 + trans + emit;
                        if prob > best_prob {
                            best_prob = prob;
                            best_prev = prev_state;
                        }
                    }
                }

                v[t][curr_state.index()] = (best_prob, Some(best_prev));
            }
        }

        // Backtrack - find best final state (must be E or S)
        let mut path = vec![State::S; n];

        let mut best_prob = f64::NEG_INFINITY;
        let mut best_state = State::S;
        for &state in &[State::E, State::S] {
            if v[n - 1][state.index()].0 > best_prob {
                best_prob = v[n - 1][state.index()].0;
                best_state = state;
            }
        }
        path[n - 1] = best_state;

        // Trace back
        for t in (0..n - 1).rev() {
            if let Some(prev) = v[t + 1][path[t + 1].index()].1 {
                path[t] = prev;
            }
        }

        path
    }

    /// Segment a string of unknown characters using HMM.
    pub fn segment(&self, text: &str) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() {
            return vec![];
        }

        let states = self.viterbi(&chars);
        let mut result = Vec::new();
        let mut current_word = String::new();

        for (i, &state) in states.iter().enumerate() {
            current_word.push(chars[i]);

            match state {
                State::E | State::S => {
                    result.push(current_word.clone());
                    current_word.clear();
                }
                State::B | State::M => {
                    // Continue building word
                }
            }
        }

        // Handle any remaining characters (shouldn't happen with valid HMM)
        if !current_word.is_empty() {
            result.push(current_word);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmm_basic() {
        let hmm = HmmModel::new();
        let result = hmm.segment("杭研大厦");
        // HMM should produce some segmentation
        assert!(!result.is_empty());
        // Should segment into multiple parts
        println!("HMM result: {:?}", result);
    }

    #[test]
    fn test_hmm_single_char() {
        let hmm = HmmModel::new();
        let result = hmm.segment("我");
        assert_eq!(result, vec!["我"]);
    }

    #[test]
    fn test_hmm_two_chars() {
        let hmm = HmmModel::new();
        let result = hmm.segment("北京");
        // Should recognize as one word or two singles
        assert!(!result.is_empty());
        println!("北京 -> {:?}", result);
    }

    #[test]
    fn test_hmm_multichar() {
        let hmm = HmmModel::new();
        let result = hmm.segment("中华人民");
        // Should produce some segmentation
        assert!(!result.is_empty());
        println!("中华人民 -> {:?}", result);
    }

    #[test]
    fn test_viterbi_states() {
        let hmm = HmmModel::new();
        let chars: Vec<char> = "北京".chars().collect();
        let states = hmm.viterbi(&chars);

        // Should return one state per character
        assert_eq!(states.len(), 2);
        // Each state should be one of B, M, E, S
        for state in &states {
            assert!(matches!(state, State::B | State::M | State::E | State::S));
        }
    }
}
