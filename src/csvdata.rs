use std::cmp::{max, min};
use std::collections::btree_map::BTreeMap;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CsvData {
    data: Vec<String>,
    delimiter: char,
    line_width: usize,
}

impl CsvData {
    pub fn new(data: Vec<String>, delimiter: char, line_width: usize) -> Self {
        CsvData {
            data,
            delimiter,
            line_width,
        }
    }

    pub fn from_raw_string(data: String, delimiter: char, line_width: usize) -> Self {
        if data.is_empty() {
            return CsvData {
                data: Vec::new(),
                delimiter,
                line_width,
            };
        }
        let mut vec: Vec<String> = data
            .split(delimiter)
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        for _ in 0..vec.len() % line_width {
            vec.push(" ".to_string());
        }

        CsvData {
            data: vec,
            delimiter,
            line_width,
        }
    }

    pub fn to_file(&self, file_name: String) -> std::io::Result<()> {
        let mut file = File::create(file_name)?;

        let vec_buf: Vec<u8> = self
            .into_iter()
            .map(|s| s.join(self.delimiter.to_string().as_ref()) + "\n")
            .map(|s| s.into_bytes())
            .flatten()
            .collect();

        let buf: &[u8] = &vec_buf;

        file.write_all(buf)
    }

    pub fn from_file<S: AsRef<str>>(filename: S, delimiter: char) -> Result<Self, Box<dyn Error>> {
        match fs::read_to_string(filename.as_ref()) {
            Ok(file) => {
                let mut line_width = 0;
                let lines: Vec<String> = file
                    .split('\n')
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                let csv_data = lines
                    .iter()
                    .flat_map(|v| {
                        line_width = max(line_width, v.matches(delimiter).count() + 1);
                        v.split(delimiter)
                    })
                    .map(|s| s.to_string())
                    .collect();

                Ok(CsvData {
                    data: csv_data,
                    delimiter,
                    line_width,
                })
            }

            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn union(&self, second: &CsvData) -> Option<CsvData> {
        if self.delimiter != second.delimiter {
            return None;
        }

        let width = max(self.line_width, second.line_width);
        let mut lines_map: BTreeMap<String, i32> = self.lines_map_from_csv(width);

        second.into_iter().for_each(|v| {
            let mut line = v.join(&self.delimiter.to_string());
            let abs = (v.len() as i32 - width as i32).abs();

            for _ in 0..abs {
                line += ", ";
            }

            *lines_map.entry(line).or_insert(0) += 1;
        });

        let mut lines = Vec::new();
        lines_map.iter().for_each(|(k, v): (&String, &i32)| {
            for _ in 0..*v {
                lines.push(k.to_owned());
            }
        });
        let result_data = lines
            .into_iter()
            .flat_map(|v| {
                v.split(self.delimiter)
                    .map(|s| s.to_owned())
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<String>>();

        Some(CsvData {
            data: result_data,
            delimiter: self.delimiter,
            line_width: width,
        })
    }

    pub fn intersection(&self, second: &CsvData) -> Option<CsvData> {
        if self.delimiter != second.delimiter {
            return None;
        }

        let width = max(self.line_width, second.line_width);
        let self_lines_map: BTreeMap<String, i32> = self.lines_map_from_csv(width);
        let second_lines_map: BTreeMap<String, i32> = second.lines_map_from_csv(width);

        let result_data = self_lines_map
            .iter()
            .filter(|(line, &_v)| second_lines_map.contains_key(*line))
            .flat_map(|(line, &v)| {
                let num_lines = min(v, *second_lines_map.get(line).unwrap());
                let mut data = Vec::new();
                for _ in 0..num_lines {
                    data.push(
                        line.split(self.delimiter)
                            .map(|s| s.to_owned())
                            .collect::<Vec<String>>(),
                    );
                }
                data
            })
            .flatten()
            .collect();

        Some(CsvData {
            data: result_data,
            delimiter: self.delimiter,
            line_width: width,
        })
    }

    pub fn difference(&self, second: &CsvData) -> Option<CsvData> {
        if self.delimiter != second.delimiter {
            return None;
        }

        let width = max(self.line_width, second.line_width);
        let self_lines_map: BTreeMap<String, i32> = self.lines_map_from_csv(width);
        let second_lines_map: BTreeMap<String, i32> = second.lines_map_from_csv(width);

        let result_data_first =
            lines_map_to_difference(&self_lines_map, &second_lines_map, &self.delimiter);
        let result_data_second =
            lines_map_to_difference(&second_lines_map, &self_lines_map, &self.delimiter);

        let mut result_data = Vec::new();
        result_data.extend(result_data_first);
        result_data.extend(result_data_second);
        Some(CsvData {
            data: result_data,
            delimiter: self.delimiter,
            line_width: width,
        })
    }

    fn lines_map_from_csv(&self, width: usize) -> BTreeMap<String, i32> {
        self.into_iter().fold(BTreeMap::new(), |mut acc, v| {
            let mut line = v.join(&self.delimiter.to_string());
            let abs = (v.len() as i32 - width as i32).abs();
            for _ in 0..abs {
                line += ", ";
            }

            *acc.entry(line).or_insert(0) += 1;
            acc
        })
    }
}

fn lines_map_to_difference(
    map1: &BTreeMap<String, i32>,
    map2: &BTreeMap<String, i32>,
    delimiter: &char,
) -> Vec<String> {
    map1.iter()
        .filter(|(line, &_v)| !map2.contains_key(*line))
        .flat_map(|(line, &num_lines)| {
            let mut data = Vec::new();
            for _ in 0..num_lines {
                data.push(
                    line.split(*delimiter)
                        .map(|s| s.to_owned())
                        .collect::<Vec<String>>(),
                );
            }
            data
        })
        .flatten()
        .collect()
}

pub fn union_all(csvs: &[CsvData], delimiter: char, line_width: usize) -> CsvData {
    let mut result_data = Vec::new();

    let mut width = csvs.iter().map(|csv| csv.line_width).max().unwrap();

    width = max(line_width, width);
    let csvs = pad(csvs, width);
    csvs.into_iter()
        .for_each(|csv| csv.into_iter().for_each(|line| result_data.extend(line)));

    CsvData {
        data: result_data,
        delimiter,
        line_width,
    }
}

pub fn intersection_all(csvs: &[CsvData]) -> Option<CsvData> {
    let mut width = csvs.iter().map(|csv| csv.line_width).max().unwrap();
    let csvs = pad(csvs, width);
    let mut csv_iterator = csvs.iter().cloned();
    let first = csv_iterator.next().unwrap();

    csv_iterator.try_fold(first, |item, other| {
        let intersection = item.intersection(&other);

        if let Some(result) = intersection {
            if result.data.is_empty() {
                return None;
            }
            return Some(result);
        }
        None
    })
}

fn difference_all(csvs: &[CsvData]) -> CsvData {
    let mut lines = BTreeMap::new();
    let mut count_map = HashMap::new();
    let width = csvs.iter().map(|csv| csv.line_width).max().unwrap();
    let csvs = pad(csvs, width);
    let length = csvs.len();
    let delim = csvs[0].delimiter;
    csvs.into_iter().enumerate().for_each(|(i, csv)| {
        csv.into_iter().for_each(|line| {
            lines
                .entry(line.clone())
                .or_insert_with(|| (0..length).into_iter().map(|_| "0").collect::<String>())
                .replace_range(i..i + 1, "1");
            *count_map.entry(line).or_insert(0) += 1;
        })
    });

    let result = lines
        .iter()
        .filter_map(|(key, value)| match num_ones(value) {
            true => {
                let mut all_lines = Vec::new();
                for _ in 0..*count_map.get(key).unwrap() as usize {
                    all_lines.extend(key.clone());
                }

                Some(all_lines)
            }
            _ => None,
        })
        .flatten()
        .collect();

    CsvData {
        data: result,
        delimiter: delim,
        line_width: width,
    }
}

fn pad(csvs: &[CsvData], line_width: usize) -> Vec<CsvData> {
    csvs.iter()
        .map(|csv| {
            let mut new_data = Vec::new();
            csv.into_iter().for_each(|data| {
                let abs = line_width - data.len();
                let mut res: Vec<String> = Vec::new();
                for item in data.iter() {
                    res.push(item.clone());
                }
                for _ in 0..abs {
                    res.push("".to_string());
                }

                new_data.extend(res)
            });
            CsvData {
                data: new_data,
                delimiter: csv.delimiter,
                line_width,
            }
        })
        .collect()
}

fn num_ones(str: &String) -> bool {
    str.chars().filter(|c| c.to_string() == "1").count() == 1
}

impl IntoIterator for CsvData {
    type Item = Vec<String>;
    type IntoIter = CsvDataIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        CsvDataIntoIterator {
            csv_data: self,
            index: 0,
        }
    }
}

pub struct CsvDataIntoIterator {
    csv_data: CsvData,
    index: usize,
}
impl Iterator for CsvDataIntoIterator {
    type Item = Vec<String>;
    fn next(&mut self) -> Option<Vec<String>> {
        if self.index >= self.csv_data.data.len() {
            return None;
        }

        let right_bound = if self.index + self.csv_data.line_width < self.csv_data.data.len() {
            self.index + self.csv_data.line_width
        } else {
            self.csv_data.data.len()
        };
        let result_slice = &self.csv_data.data[self.index..right_bound];
        let result: Vec<String> = result_slice.to_vec();
        self.index += self.csv_data.line_width;
        Some(result)
    }
}

impl<'a> IntoIterator for &'a CsvData {
    type Item = Vec<String>;
    type IntoIter = CsvDataIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CsvDataIterator {
            csv_data: self,
            index: 0,
        }
    }
}
pub struct CsvDataIterator<'a> {
    csv_data: &'a CsvData,
    index: usize,
}

impl<'a> Iterator for CsvDataIterator<'a> {
    type Item = Vec<String>;
    fn next(&mut self) -> Option<Vec<String>> {
        if self.index >= self.csv_data.data.len() {
            return None;
        }

        let right_bound = if self.index + self.csv_data.line_width < self.csv_data.data.len() {
            self.index + self.csv_data.line_width
        } else {
            self.csv_data.data.len()
        };
        let result_slice = &self.csv_data.data[self.index..right_bound];
        let result: Vec<String> = result_slice.to_vec();
        self.index += self.csv_data.line_width;
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::csvdata::{difference_all, intersection_all, pad, union_all, CsvData};
    use std::fs;

    #[test]
    fn test_from_str() {
        let expect = vec!["test", "test2", "test3"];
        let tmp = CsvData::from_raw_string("test,test2,test3".to_string(), ',', 1);
        assert_eq!(tmp.data, expect);
    }

    #[test]
    fn test_from_str_extra() {
        let expect = vec!["test", "test2", "test3", "", "", "", ""];
        let tmp = CsvData::from_raw_string("test,test2,test3,,,,".to_string(), ',', 1);
        assert_eq!(tmp.data, expect);
    }

    #[test]
    fn test_from_str_fail() {
        let expect = vec!["test", "te2", "test3"];
        let tmp = CsvData::from_raw_string("test,test2,test3".to_string(), ',', 1);
        assert_ne!(tmp.data, expect);
    }

    #[test]
    fn test_iterator() {
        let expect = vec!["test", "te2", "test3"];
        let tmp = CsvData::from_raw_string("test,test2,test3".to_string(), ',', 2);

        tmp.into_iter().for_each(|x| println!("{:?}", x));
        //assert_ne!(tmp.data, expect);
    }
    #[test]
    fn test_iterator_non_consuming() {
        let expect = vec!["test", "te2", "test3"];
        let tmp = &CsvData::from_raw_string("test,test2,test3".to_string(), ',', 2);

        tmp.into_iter()
            .for_each(|x| println!("{}", x.join(&tmp.delimiter.to_string())));
        assert_ne!(tmp.data, expect);
    }

    #[test]
    fn test_write_to_file() {
        let tmp = &CsvData::from_raw_string("test,test2,test3".to_string(), ',', 2);

        fs::remove_file("testdata/test.csv");

        match tmp.to_file(String::from("testdata/test.csv")) {
            Ok(..) => assert!(true),
            Err(E) => {
                println!("{}", E);
                assert!(false)
            }
        }
    }

    #[test]
    fn test_from_file() {
        let tmp = CsvData::from_raw_string("test,test2,test3".to_string(), ',', 2);

        match CsvData::from_file("testdata/test.csv", ',') {
            Ok(data) => assert_eq!(tmp, data),
            Err(E) => assert!(false),
        }
    }

    #[test]
    fn test_from_input_output() {
        let tmp = CsvData::from_raw_string("test,test2,test3".to_string(), ',', 2);
        let tmp2 = tmp.clone();
        fs::remove_file("testdata/testinputoutput.csv");
        tmp2.to_file(String::from("testdata/testinputoutput.csv"));
        let result = CsvData::from_file("testdata/testinputoutput.csv", ',').unwrap();
        assert_eq!(tmp, result)
    }

    #[test]
    fn test_union() {
        let tmp = CsvData::from_raw_string("test,test2,test3".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test5,test6".to_string(), ',', 2);
        let expected = CsvData::from_raw_string(
            "test,test2,test,test2,test3, ,test3,test4,test5,test6".to_string(),
            ',',
            2,
        );
        let result = tmp.union(&tmp2).unwrap();
        assert_eq!(expected, result)
    }

    #[test]
    fn test_union_2() {
        let tmp = CsvData::from_raw_string("test,test2, , ,test3".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test5,test6".to_string(), ',', 2);
        let expected = CsvData::from_raw_string(
            " , ,test,test2,test,test2,test3, ,test3,test4,test5,test6".to_string(),
            ',',
            2,
        );
        let result = tmp.union(&tmp2).unwrap();
        assert_eq!(expected, result)
    }

    #[test]
    fn test_union_different_widths() {
        let tmp = CsvData::from_raw_string("test,test2,test3".to_string(), ',', 3);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test5,test6".to_string(), ',', 4);
        let expected = CsvData::from_raw_string(
            "test,test2,test3, ,test,test2,test3,test4,test5,test6, , ".to_string(),
            ',',
            4,
        );
        let result = tmp.union(&tmp2).unwrap();
        assert_eq!(expected, result)
    }

    #[test]
    fn test_union_different_widths_rev() {
        let tmp = CsvData::from_raw_string("test,test2,test3".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test5,test6".to_string(), ',', 1);
        let expected = CsvData::from_raw_string(
            "test, ,test,test2,test2, ,test3, ,test3, ,test4, ,test5, ,test6, ".to_string(),
            ',',
            2,
        );
        let result = tmp.union(&tmp2).unwrap();
        assert_eq!(expected, result)
    }
    #[test]
    fn test_intersection() {
        let tmp = CsvData::from_raw_string("test,test2,test3".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test5,test6".to_string(), ',', 2);
        let expected = CsvData::from_raw_string("test,test2".to_string(), ',', 2);
        let result = tmp.intersection(&tmp2).unwrap();
        println!("{:?} {:?}", tmp, result);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_intersection_larger() {
        let tmp = CsvData::from_raw_string("test,test2,test3,test4".to_string(), ',', 4);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test5,test6".to_string(), ',', 4);
        let expected = CsvData::from_raw_string("test,test2,test3,test4".to_string(), ',', 4);
        let result = tmp.intersection(&tmp2).unwrap();
        println!("{:?} {:?}", tmp, result);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_intersection_diff() {
        let tmp =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp2 = CsvData::from_raw_string(
            "test,test2,test3,test4,test5,test6,test3,test4".to_string(),
            ',',
            2,
        );
        let expected =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let result = tmp.intersection(&tmp2).unwrap();
        println!("{:?} {:?}", tmp, result);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_difference() {
        let tmp =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp2 = CsvData::from_raw_string(
            "test,test2,test3,test4,test5,test6,test3,test4".to_string(),
            ',',
            2,
        );
        let expected = CsvData::from_raw_string("test5,test6".to_string(), ',', 2);
        let result = tmp.difference(&tmp2).unwrap();
        println!("{:?} {:?}", tmp, result);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_difference_empty() {
        let tmp =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let expected = CsvData::from_raw_string(String::new(), ',', 2);
        let result = tmp.difference(&tmp2).unwrap();
        println!("{:?} {:?}", tmp, result);
        assert_eq!(expected, result)
    }

    #[test]
    fn test_union_all() {
        let tmp =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp3 =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);

        let mut vec = vec![tmp, tmp2, tmp3];

        let expected = CsvData::from_raw_string("test,test2,test3,test4,test3,test4,test,test2,test3,test4,test3,test4,test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let result = union_all(&vec, ',', 2);

        assert_eq!(expected, result)
    }

    #[test]
    fn test_union_all_change_width() {
        let tmp =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp3 =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);

        let vec = vec![tmp, tmp2, tmp3];

        let expected = CsvData::from_raw_string("test,test2,,,test3,test4,,,test3,test4,,,test,test2,,,test3,test4,,,test3,test4,,,test,test2,,,test3,test4,,,test3,test4,,".to_string(), ',', 4);
        let result = union_all(&vec, ',', 4);

        assert_eq!(expected, result)
    }

    #[test]
    fn test_intersection_all() {
        let tmp =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp3 =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);

        let mut vec = vec![tmp, tmp2, tmp3];

        let expected =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let result = intersection_all(&vec).unwrap();

        assert_eq!(expected, result)
    }

    #[test]
    fn test_intersection_all_different_widths() {
        let tmp =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,,,test3,test4,test3,test4".to_string(), ',', 4);
        let tmp3 =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);

        let mut vec = vec![tmp, tmp2, tmp3];

        let expected = CsvData::from_raw_string("test,test2,,".to_string(), ',', 4);
        let result = intersection_all(&vec).unwrap();

        assert_eq!(expected, result)
    }

    #[test]
    fn test_intersection_all_none() {
        let tmp =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp3 = CsvData::from_raw_string("asdas,addsad".to_string(), ',', 2);

        let vec = vec![tmp, tmp2, tmp3];

        let result = intersection_all(&vec);
        println!("{:?}", result);
        assert!(result.is_none())
    }

    #[test]
    fn test_difference_all() {
        let tmp = CsvData::from_raw_string(
            "test,test2,test3,test4,test3,test4,adfas,addsad".to_string(),
            ',',
            2,
        );
        let tmp2 = CsvData::from_raw_string("test3,test4,test3,test4".to_string(), ',', 2);
        let tmp3 = CsvData::from_raw_string("adfas,addsad".to_string(), ',', 2);

        let vec = vec![tmp, tmp2, tmp3];
        let expected = CsvData::from_raw_string("test,test2".to_string(), ',', 2);
        let result = difference_all(&vec);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_difference_all_none() {
        let tmp = CsvData::from_raw_string(
            "test,test2,test3,test4,test3,test4,adfas,addsad".to_string(),
            ',',
            2,
        );
        let tmp2 = CsvData::from_raw_string("test3,test4,test3,test4".to_string(), ',', 2);
        let tmp3 = CsvData::from_raw_string("test,test2,adfas,addsad".to_string(), ',', 2);

        let vec = vec![tmp, tmp2, tmp3];
        let expected = CsvData::from_raw_string("".to_string(), ',', 2);
        let result = difference_all(&vec);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_pad() {
        let tmp =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);

        let result = pad(&[tmp, tmp2], 4);

        assert!(result
            .iter()
            .cloned()
            .all(|csv| csv.into_iter().all(|line| line.len() == 4)));
    }

    #[test]
    #[should_panic]
    fn test_pad_smaller() {
        let tmp =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);
        let tmp2 =
            CsvData::from_raw_string("test,test2,test3,test4,test3,test4".to_string(), ',', 2);

        let _result = pad(&[tmp, tmp2], 1);
    }
}
