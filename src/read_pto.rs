use std::str::FromStr;
use nom::{
    IResult,
    bytes::complete::{tag, take_while1, take_until},
    combinator::map_res,
    number::complete::double,
    character::complete::multispace0,
    multi::fold_many1,
};


#[derive(Debug, PartialEq, Clone)]
pub struct ControlPoint {
    image_id: u64,
    x_coord: f64,
    y_coord: f64,
}

impl ControlPoint {
    fn new(image_id: u64, x_coord: f64, y_coord: f64) -> ControlPoint
    {
        ControlPoint{image_id, x_coord, y_coord}
    }
}

fn uinteger64(input: &str) -> IResult<&str, u64> {

    map_res(
        take_while1(|c: char| c.is_digit(10)),
        |s| u64::from_str(s)
    )(input)
}

fn control_point_pair(input: &str) -> IResult<&str, (ControlPoint, ControlPoint)> {

    let (i, _) = tag("c")(input)?;

    let (i, _) = tag(" n")(i)?;
    let (i, id1) = uinteger64(i)?;

    let (i, _) = tag(" N")(i)?;
    let (i, id2) = uinteger64(i)?;

    let (i, _) = tag(" x")(i)?;
    let (i, x1) = double(i)?;

    let (i, _) = tag(" y")(i)?;
    let (i, y1) = double(i)?;

    let (i, _) = tag(" X")(i)?;
    let (i, x2) = double(i)?;

    let (i, _) = tag(" Y")(i)?;
    let (i, y2) = double(i)?;

    let (i, _) = tag(" t0")(i)?;

    let (i, _) = multispace0(i)?;

    Ok((i,
        (
            ControlPoint{image_id: id1, x_coord: x1, y_coord: y1},
            ControlPoint{image_id: id2, x_coord: x2, y_coord: y2}
        )
    ))
}

fn read_control_point_pairs_impl(pto_file_contents: &str) -> IResult<&str, Vec<(ControlPoint, ControlPoint)>> {

    let (i, _) = take_until("# control points")(pto_file_contents)?;
    let (i, _) = tag("# control points")(i)?;
    let (i, _) = multispace0(i)?;

    let (i, pairs) =
    fold_many1(
        control_point_pair,
        Vec::new(),
        |mut acc: Vec<_>, item| {
            acc.push(item);
            acc
        }
    )(i)?;

    Ok((i, pairs))
}


pub fn read_control_point_pairs(pto_file_contents: &str) -> std::result::Result<Vec<(ControlPoint, ControlPoint)>, String> {

    match read_control_point_pairs_impl(pto_file_contents) {
        Ok((_, v)) => Ok(v),
        Err(nom::Err::Error(e)) => Err(format!("{:?}", e)),
        Err(nom::Err::Incomplete(e)) => Err(format!("{:?}", e)),
        Err(nom::Err::Failure(e)) => Err(format!("{:?}", e)),
    }
}


#[cfg(test)]
mod test {
    use super::*;

    use assert_matches::*;

    #[test]
    fn uinteger64_test() {
        assert_eq!(uinteger64("0"), Ok(("", 0)));
        assert_eq!(uinteger64("123456789"), Ok(("", 123456789)));
        assert_eq!(uinteger64("12345678a"), Ok(("a", 12345678)));

        assert_matches!(uinteger64("-1"), Err(_));
        assert_matches!(uinteger64(""), Err(_));

        assert_eq!(uinteger64("123 456 789"), Ok((" 456 789", 123)));
    }

    #[test]
    fn control_point_pair_test() {
        {
            let (s, (cp1, cp2)) =
                control_point_pair("c n123 N456 x789 y876 X543 Y210 t0").unwrap();

            assert_eq!(s, "");
            assert_eq!(cp1, ControlPoint::new(123, 789 as f64, 876 as f64));
            assert_eq!(cp2, ControlPoint::new(456, 543 as f64, 210 as f64));
        }

        {
            let (s, (cp1, cp2)) =
                control_point_pair("c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0").unwrap();

            assert_eq!(s, "");
            assert_eq!(cp1, ControlPoint::new(0, 568.542826048136, 117.691966641595));
            assert_eq!(cp2, ControlPoint::new(1, 54.4570607766205, 98.7300002744364));
        }

        assert_matches!(control_point_pair("c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0"), Ok(_));
        assert_matches!(control_point_pair("c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0 "), Ok(_));
        assert_matches!(control_point_pair("c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0\t"), Ok(_));
        assert_matches!(control_point_pair("c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0\n "), Ok(_));
        assert_matches!(control_point_pair("c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0trailing text"), Ok(_));
        assert_matches!(control_point_pair( " n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0"), Err(_));
        assert_matches!(control_point_pair("c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364"   ), Err(_));
        assert_matches!(control_point_pair("c n0 n1 x568.542826048136 y117.691966641595 x54.4570607766205 y98.7300002744364 t0"), Err(_));
    }

    #[test]
    fn read_control_point_pairs_test() {

        //one control point pair
        {
let pto_file_contents =
"file contents

# control points
c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0

more file contents
";
            let v = read_control_point_pairs(pto_file_contents).unwrap();
            assert_eq!(1, v.len());
            assert_eq!(v[0].0, ControlPoint::new(0, 568.542826048136, 117.691966641595));
            assert_eq!(v[0].1, ControlPoint::new(1, 54.4570607766205, 98.7300002744364));
        }

        //3 control point pairs
        {
let pto_file_contents =
"file contents

# control points
c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0
c n1 N123 x111.1 y222.2 X333.3 Y444.4 t0
c n2 N123 x555.5 y222.2 X333.3 Y444.4 t0

more file contents
";
            let v = read_control_point_pairs(pto_file_contents).unwrap();
            assert_eq!(3, v.len());
            assert_eq!(v[0].0, ControlPoint::new(0, 568.542826048136, 117.691966641595));
            assert_eq!(v[0].1, ControlPoint::new(1, 54.4570607766205, 98.7300002744364));
            assert_eq!(v[1].0, ControlPoint::new(1, 111.1, 222.2));
            assert_eq!(v[1].1, ControlPoint::new(123, 333.3, 444.4));
            assert_eq!(v[2].0, ControlPoint::new(2, 555.5, 222.2));
            assert_eq!(v[2].1, ControlPoint::new(123, 333.3, 444.4));
        }

        //no control point pairs
        {
let pto_file_contents =
"file contents

# control points

more file contents
";
            assert_matches!(read_control_point_pairs(pto_file_contents), Err(_));
        }

        //no control point section header
        {
let pto_file_contents =
"file contents

c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0

more file contents
";
            assert_matches!(read_control_point_pairs(pto_file_contents), Err(_));
        }

        //malformed control point pair line
        {
let pto_file_contents =
"file contents

# control points
c n0 N1 x568.542826048136 y117.691966641595 [invalid] X54.4570607766205 Y98.7300002744364 t0

more file contents
";
            assert_matches!(read_control_point_pairs(pto_file_contents), Err(_));
        }
    }
}