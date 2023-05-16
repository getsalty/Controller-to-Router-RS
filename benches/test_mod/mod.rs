pub mod challenge {
    fn convert_char_to_ascii_value(c: char) -> u32 {
        let tmp_val = c as u32;
        if tmp_val >= 65 && tmp_val <= 90 {
            tmp_val - 38
        } else if tmp_val >= 97 && tmp_val <= 122 {
            tmp_val - 96
        } else {
            0
        }
    }

    pub fn load_input() -> Vec<(Vec<u32>, Vec<u32>)> {
        let input = include_str!("input.txt");
        input
            .lines()
            .map(|l| {
                let length = l.to_string().len();
                let first_half = &l[0..(length / 2)];
                let second_half = &l[(length / 2)..length];

                (first_half, second_half)
            })
            .map(|(first_half, second_half)| {
                let first_half = first_half
                    .clone()
                    .chars()
                    .map(|c| convert_char_to_ascii_value(c))
                    .collect::<Vec<u32>>();
                let second_half = second_half
                    .clone()
                    .chars()
                    .map(|c| convert_char_to_ascii_value(c))
                    .collect::<Vec<u32>>();

                (first_half, second_half)
            })
            .collect()
    }

    pub fn part_one_a(input: &Vec<(Vec<u32>, Vec<u32>)>) -> u32 {
        let mut result = 0;

        for rucksack in input {
            let (first_half, second_half) = &rucksack;

            let mut first_half = first_half.clone();
            let mut second_half = second_half.clone();

            first_half.sort();
            second_half.sort();

            let common_value = find_the_common_value(first_half, second_half);

            if let Some(common_value) = common_value {
                result += common_value;
            }
        }

        result
    }

    pub fn part_one_b(input: &Vec<(Vec<u32>, Vec<u32>)>) -> u32 {
        let mut result = 0;

        for rucksack in input {
            let (first_half, second_half) = &rucksack;

            let mut first_half = first_half.clone();
            let mut second_half = second_half.clone();

            first_half.sort();
            second_half.sort();

            let common_value = find_the_common_value_two(first_half, second_half);

            if let Some(common_value) = common_value {
                result += common_value;
            }
        }

        result
    }

    pub fn part_one_c(input: &Vec<(Vec<u32>, Vec<u32>)>) -> u32 {
        let mut result = 0;

        for rucksack in input {
            let (first_half, second_half) = &rucksack;

            let mut first_half = first_half.clone();
            let mut second_half = second_half.clone();

            first_half.sort();
            second_half.sort();

            let common_value = find_the_common_value_three(first_half, second_half);

            if let Some(common_value) = common_value {
                result += common_value;
            }
        }

        result
    }

    pub fn part_one_d(input: &Vec<(Vec<u32>, Vec<u32>)>) -> u32 {
        let mut result = 0;

        for rucksack in input {
            let (first_half, second_half) = &rucksack;

            let mut first_half = first_half.clone();
            let mut second_half = second_half.clone();

            first_half.sort();
            second_half.sort();

            let common_value = find_the_common_value_four(first_half, second_half);

            if let Some(common_value) = common_value {
                result += common_value;
            }
        }

        result
    }

    fn find_the_common_value(first_half: Vec<u32>, second_half: Vec<u32>) -> Option<u32> {
        let mut second_half = second_half.clone();

        for i in 0..first_half.len() {
            let mut j = 0;
            while j < second_half.len() {
                if first_half[i] == second_half[j] {
                    return Some(first_half[i]);
                }

                if first_half[i] > second_half[j] {
                    second_half.remove(j);
                } else {
                    j += 1;
                }
            }
        }

        None
    }

    fn find_the_common_value_two(first_half: Vec<u32>, second_half: Vec<u32>) -> Option<u32> {
        for i in 0..first_half.len() {
            for j in 0..first_half.len() {
                if first_half[i] == second_half[j] {
                    return Some(first_half[i]);
                }
            }
        }

        None
    }

    fn find_the_common_value_three(first_half: Vec<u32>, second_half: Vec<u32>) -> Option<u32> {
        let mut j = 0;

        for i in 0..first_half.len() {
            while j < first_half.len() {
                if first_half[i] == second_half[j] {
                    return Some(first_half[i]);
                } else if first_half[i] > second_half[j] {
                    j += 1;
                } else if first_half[i] < second_half[j] {
                    break;
                }
            }
        }

        None
    }

    fn find_the_common_value_four(first_half: Vec<u32>, second_half: Vec<u32>) -> Option<u32> {
        for i in 0..first_half.len() {
            if second_half.binary_search(&first_half[i]).is_ok() {
                return Some(first_half[i]);
            };
        }

        None
    }
}
