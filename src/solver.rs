use std::cmp::Ordering;
use std::collections::HashMap;
use std::vec;
use crate::util::{is_part_of, new_vec, union_point};

#[derive(Debug, Clone)]
pub struct Solution {
    pub steps: Vec<Step>,
    pub conditions: Vec<bool>,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Step {
    pub x: u8,
    pub y: u8,
}

impl Step {
    pub fn new(x: u8, y: u8) -> Self {
        Step { x, y }
    }
}

/// The shortest solutions for each `condition` combinations.
/// - First solutions cover first(shortest) single conditions.
/// - Last solutions cover as much as possible last(longest) conditions.
pub fn filter_best(all_solutions: &Vec<Solution>) -> Vec<Solution> {
    let mut map: HashMap<&Vec<bool>, &Solution> = HashMap::new();
    for solution in all_solutions {
        match map.get(&solution.conditions) {
            Some(existed) => {
                if existed.steps.len() > solution.steps.len() {
                    map.insert(&solution.conditions, solution);
                }
            }
            None => { map.insert(&solution.conditions, solution); }
        };
    }

    let mut result = map.into_values().map(|s| s.clone()).collect::<Vec<_>>();
    result.sort_by(|s1, s2| {
        let score1 = s1.conditions.iter().filter(|c| **c).count();
        let score2 = s2.conditions.iter().filter(|c| **c).count();
        if score1 > score2 {
            return Ordering::Greater;
        } else if score1 < score2 {
            return Ordering::Less;
        }

        for (score1, score2) in s1.conditions.iter().zip(&s2.conditions).rev() {
            if *score1 && !*score2 {
                return Ordering::Greater;
            } else if !*score1 && *score2 {
                return Ordering::Less;
            }
        }
        return Ordering::Equal;
    });
    return result;
}

/// Find all unique solutions. Use `filter_best` to filter and sort them.
pub fn solve(matrix: &Vec<Vec<u8>>, conditions: &Vec<Vec<u8>>, step_limit: usize) -> Vec<Solution> {
    // Each solution covers single condition. Not finalized.
    let mut solutions: Vec<Solution> = conditions.iter().enumerate().flat_map(|(cond_i, cond)| {
        let mut conds = Vec::with_capacity(conditions.len());
        conds.resize(conditions.len(), false);
        conds[cond_i] = true;
        find_condition_solutions(cond, None, &matrix).into_iter().map(move |solution| {
            Solution { steps: solution, conditions: conds.clone() }
        })
    }).collect();

    // Each solution may cover several conditions. But still not finalized.
    merge_solutions(&mut solutions, step_limit);

    return solutions.into_iter().filter_map(|s| {
        finalize_solution(&s.steps, &matrix, step_limit)
            .map(move |finalized| Solution { steps: finalized, conditions: s.conditions })
    }).collect();
}

/// Merges solutions.
/// - if `b` solution is small piece of `a` solution - corrects `a` conditions. `a` solution covers `a+b` conditions;
/// - if `b` solution's start is ending of `a` solution OR `a` and `b` have no same steps - tries
/// to add new `a+b` solution which covers `a+b` conditions (maybe with additional steps between `a` and `b`);
/// - all new solutions must not be greater than `step_limit`
///
/// **It is not finalized**
fn merge_solutions(solutions: &mut Vec<Solution>, step_limit: usize) {
    let mut changed = true;
    while changed {
        changed = false;

        for i in 0..solutions.len() {
            let mut src = solutions.get(i).unwrap().clone();

            for j in 0..solutions.len() {
                if i == j {
                    continue;
                }

                let dest = solutions.get(j).unwrap();

                if src.conditions == dest.conditions {
                    // same conditions. Useless for merging
                    continue;
                }

                if is_part_of(&src.steps, &dest.steps) {
                    // `dest` is a small part of `src`. `dest` conditions are `src`
                    for (src_cond, dest_cond) in src.conditions.iter_mut().zip(&dest.conditions) {
                        if !*src_cond && *dest_cond {
                            *src_cond = true;
                            changed = true;
                        }
                    }
                } else if let Some(p) = union_point(&src.steps, &dest.steps) {
                    // `dest` is ending of `src`
                    if src.steps[..p].iter().any(|s| dest.steps.contains(s)) {
                        // step intersections. No solutions
                        continue;
                    }

                    let total_steps = p + dest.steps.len();
                    if total_steps > step_limit {
                        continue;
                    }

                    let good_direction = if src.steps.len() - p > 1 {
                        true // >=2 steps are same. We may not check horizontal/vertical logic here
                    } else {
                        // 1 last step is same. Need to check horizontal/vertical logic
                        src.steps.len() <= 1 || dest.steps.len() <= 1 ||
                            is_horizontal_step(&src.steps[src.steps.len() - 2], &src.steps[src.steps.len() - 1]) != is_horizontal_step(&dest.steps[0], &dest.steps[1])
                    };
                    if good_direction {
                        let solution = new_vec(&src.steps[..p], &dest.steps);
                        if solutions.iter().all(|v| v.steps != solution) {
                            let conds = src.conditions.iter().zip(&dest.conditions).map(|(a, b)| *a || *b).collect();
                            solutions.push(Solution { steps: solution, conditions: conds });
                            changed = true;
                        }
                    } else {
                        // steps are incompatible. No common solutions
                    }
                } else if src.steps.iter().all(|s| !dest.steps.contains(s)) {
                    // steps has no intersections. May be merged
                    if src.steps.len() + dest.steps.len() > step_limit {
                        continue;
                    }

                    let last = src.steps.last().unwrap();
                    let first = dest.steps.first().unwrap();
                    let no_additional_steps = if last.x == first.x {
                        (src.steps.len() <= 1 || is_horizontal_step(&src.steps[src.steps.len() - 2], &src.steps[src.steps.len() - 1])) &&
                            (dest.steps.len() <= 1 || is_horizontal_step(&dest.steps[0], &dest.steps[1]))
                    } else if last.y == first.y {
                        (src.steps.len() <= 1 || !is_horizontal_step(&src.steps[src.steps.len() - 2], &src.steps[src.steps.len() - 1])) &&
                            (dest.steps.len() <= 1 || !is_horizontal_step(&dest.steps[0], &dest.steps[1]))
                    } else {
                        false
                    };
                    if no_additional_steps {
                        let solution = new_vec(&src.steps, &dest.steps);
                        if solutions.iter().all(|v| v.steps != solution) {
                            let conds = src.conditions.iter().zip(&dest.conditions).map(|(a, b)| *a || *b).collect();
                            solutions.push(Solution { steps: solution, conditions: conds });
                            changed = true;
                        }
                    }

                    // TODO: Solution with additional steps may be possible, but I'm tired.
                    //       Usually this case is filtered by `step_limit` restriction.
                    //       Those solutions will be too large.
                }
            }

            solutions[i] = src;
        }
    }
}

/// Step chains, which covers single `condition`.
/// **It is not finalized**
fn find_condition_solutions(condition: &[u8], steps: Option<Vec<Step>>, matrix: &Vec<Vec<u8>>) -> Vec<Vec<Step>> {
    let steps = steps.unwrap_or_default();

    // solution meets condition
    if condition.is_empty() {
        return vec![steps];
    }

    // next step covers next condition item. It shifts condition items
    let next_hex = condition[0];
    let next_steps = next_possible_steps(&steps, next_hex, &matrix);
    return next_steps.into_iter().flat_map(|step| {
        let new_steps = new_vec(&steps, &[step]);
        find_condition_solutions(&condition[1..], Some(new_steps), &matrix)
    }).collect();
}

/// Solution must start with `y=0` and be vertical. Tries to do it.
/// It may require 0-3 additional steps. But each solution must be not greater than `step_limit`
fn finalize_solution(s: &[Step], matrix: &Vec<Vec<u8>>, step_limit: usize) -> Option<Vec<Step>> {
    if s.is_empty() {
        return None;
    }

    if s[0].y == 0 {
        if s.len() == 1 {
            return Some(Vec::from(s)); // single step solution
        }
        if !is_horizontal_step(&s[0], &s[1]) {
            return Some(Vec::from(s)); // steps are finalized
        }
    }

    if s.len() == 1 {
        // 1 additional step
        return if step_limit >= s.len() + 1 {
            Some(vec![Step::new(s[0].x, 0), s[0]])
        } else {
            None
        };
    }

    let width = matrix[0].len() as u8;
    let height = matrix.len() as u8;

    if is_horizontal_step(&s[0], &s[1]) {
        let first_step = Step::new(s[0].x, 0);
        if !s.contains(&first_step) {
            // 1 additional step
            return if step_limit >= s.len() + 1 {
                Some(new_vec(&[first_step], &s))
            } else {
                None
            };
        }

        // hard-way. 3 additional steps are needed
        if step_limit < s.len() + 3 {
            return None;
        }
        for x in 0..width {
            if x == s[0].x {
                continue;
            }

            for y in 0..height {
                if y == s[0].y {
                    continue;
                }

                let first = Step::new(x, 0);
                let second = Step::new(x, y);
                let third = Step::new(s[0].x, y);

                if s.contains(&first) || s.contains(&second) || s.contains(&third) {
                    continue;
                }

                return Some(new_vec(&[first, second, third], &s));
            }
        }
        return None;
    } else {
        // 2 additional steps are needed
        if step_limit < s.len() + 2 {
            return None;
        }
        for x in 0..width {
            if x == s[0].x {
                continue;
            }

            let first = Step::new(x, 0);
            let second = Step::new(x, s[0].y);
            if s.contains(&first) || s.contains(&second) {
                continue;
            }

            return Some(new_vec(&[first, second], &s));
        }
        return None;
    }
}

/// Find next possible steps to cover `next_code`.
/// Respects vertical/horizontal logic and previous steps.
fn next_possible_steps(steps: &Vec<Step>, next_code: u8, matrix: &Vec<Vec<u8>>) -> Vec<Step> {
    let all_possible_steps: Vec<Step> = matrix.iter().enumerate().flat_map(move |(y, line)| {
        line.iter().enumerate().filter_map(move |(x, hex)| {
            let step = Step::new(x as u8, y as u8);
            if *hex == next_code && !steps.contains(&step) {
                Some(step)
            } else {
                None
            }
        })
    }).collect();
    if all_possible_steps.is_empty() {
        return all_possible_steps;
    }


    let last_step = steps.last();
    if last_step.is_none() {
        // first step
        return all_possible_steps;
    }
    let last_step = last_step.unwrap();

    return match next_step_is_horizontal(&steps) {
        None => all_possible_steps.into_iter().filter(|s| s.x == last_step.x || s.y == last_step.y).collect(), // second step
        Some(true) => all_possible_steps.into_iter().filter(|s| s.y == last_step.y).collect(),
        Some(false) => all_possible_steps.into_iter().filter(|s| s.x == last_step.x).collect(),
    };
}


fn next_step_is_horizontal(steps: &[Step]) -> Option<bool> {
    return if steps.len() <= 1 {
        None // first step does not matter
    } else {
        Some(!is_horizontal_step(&steps[steps.len() - 2], &steps[steps.len() - 1]))
    };
}

#[inline]
fn is_horizontal_step(s1: &Step, s2: &Step) -> bool {
    s1.x != s2.x
}


#[cfg(test)]
mod tests {
    use crate::solver::{finalize_solution, find_condition_solutions, is_horizontal_step, next_possible_steps, next_step_is_horizontal, Step};

    #[test]
    fn test_is_horizontal_step() {
        assert_eq!(true, is_horizontal_step(&Step::new(0, 0), &Step::new(2, 0)));
        assert_eq!(false, is_horizontal_step(&Step::new(0, 0), &Step::new(0, 2)));
    }

    #[test]
    fn test_next_step_is_horizontal() {
        assert_eq!(Some(true), next_step_is_horizontal(&vec![Step::new(0, 0), Step::new(0, 2)]));
        assert_eq!(Some(false), next_step_is_horizontal(&vec![Step::new(0, 0), Step::new(2, 0)]));
        assert_eq!(None, next_step_is_horizontal(&vec![Step::new(0, 0)]));
        assert_eq!(None, next_step_is_horizontal(&vec![]));
    }

    #[test]
    fn test_next_possible_steps() {
        let matrix = vec![
            vec![2, 1, 2],
            vec![2, 2, 2],
            vec![1, 1, 2],
        ];

        // first step
        assert_eq!(
            vec![Step::new(1, 0), Step::new(0, 2), Step::new(1, 2)],
            next_possible_steps(&vec![], 1, &matrix),
        );

        // second step
        assert_eq!(
            vec![Step::new(2, 0), Step::new(0, 1)],
            next_possible_steps(&vec![Step::new(0, 0)], 2, &matrix),
        );

        // third step
        assert_eq!(
            vec![Step::new(1, 1), Step::new(2, 1)],
            next_possible_steps(&vec![Step::new(0, 0), Step::new(0, 1)], 2, &matrix),
        );

        // no steps
        assert_eq!(
            Vec::<Step>::new(),
            next_possible_steps(&vec![Step::new(0, 0), Step::new(0, 1), Step::new(1, 1)], 2, &matrix),
        );
    }

    #[test]
    fn test_finalize_solution() {
        // 3x3 any matrix
        let matrix = vec![
            vec![1, 1, 1],
            vec![],
            vec![],
        ];

        assert_eq!(None, finalize_solution(&vec![], &matrix, 1));

        // solution is already finalized
        assert_eq!(Some(vec![Step::new(0, 0)]), finalize_solution(&vec![Step::new(0, 0)], &matrix, 1));
        assert_eq!(Some(vec![Step::new(0, 0), Step::new(0, 1)]), finalize_solution(&vec![Step::new(0, 0), Step::new(0, 1)], &matrix, 2));

        // 1 additional steps
        assert_eq!(Some(vec![Step::new(0, 0), Step::new(0, 1)]), finalize_solution(&vec![Step::new(0, 1)], &matrix, 2));
        assert_eq!(None, finalize_solution(&vec![Step::new(0, 1)], &matrix, 1));

        assert_eq!(Some(vec![Step::new(0, 0), Step::new(0, 1), Step::new(1, 1)]), finalize_solution(&vec![Step::new(0, 1), Step::new(1, 1)], &matrix, 3));
        assert_eq!(None, finalize_solution(&vec![Step::new(0, 1), Step::new(1, 1)], &matrix, 2));

        // 2 additional steps
        assert_eq!(
            Some(vec![Step::new(0, 0), Step::new(0, 1), Step::new(1, 1), Step::new(1, 2), Step::new(2, 2)]),
            finalize_solution(&vec![Step::new(1, 1), Step::new(1, 2), Step::new(2, 2)], &matrix, 5),
        );
        assert_eq!(
            None,
            finalize_solution(&vec![Step::new(1, 1), Step::new(1, 2), Step::new(2, 2)], &matrix, 4),
        );

        // 3 additional steps
        assert_eq!(
            Some(vec![Step::new(0, 0), Step::new(0, 2), Step::new(1, 2), Step::new(1, 1), Step::new(2, 1), Step::new(2, 0), Step::new(1, 0)]),
            finalize_solution(&vec![Step::new(1, 1), Step::new(2, 1), Step::new(2, 0), Step::new(1, 0)], &matrix, 7),
        );
        assert_eq!(
            None,
            finalize_solution(&vec![Step::new(1, 1), Step::new(2, 1), Step::new(2, 0), Step::new(1, 0)], &matrix, 6),
        );
    }

    #[test]
    fn test_find_condition_solutions() {
        let matrix = vec![
            vec![0, 9, 0, 9],
            vec![9, 9, 1, 0],
            vec![1, 9, 2, 9],
            vec![9, 0, 1, 9],
        ];

        let solutions = find_condition_solutions(&vec![0, 1, 2], None, &matrix);
        assert_eq!(
            vec![
                vec![Step::new(0, 0), Step::new(0, 2), Step::new(2, 2)],
                vec![Step::new(3, 1), Step::new(2, 1), Step::new(2, 2)],
                vec![Step::new(1, 3), Step::new(2, 3), Step::new(2, 2)],
            ],
            solutions,
        );

        let no_solutions = find_condition_solutions(&vec![0, 1, 8], None, &matrix);
        assert_eq!(Vec::<Vec<Step>>::new(), no_solutions);
    }
}