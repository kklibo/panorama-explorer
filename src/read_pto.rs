use std::str::FromStr;
use nom::{
    IResult,
    bytes::complete::{tag, take_while1},
    combinator::{map_res, eof},
    number::complete::double
};


#[derive(Debug, PartialEq)]
struct ControlPoint {
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

    eof(i)?;

    Ok((i,
        (
            ControlPoint{image_id: id1, x_coord: x1, y_coord: y1},
            ControlPoint{image_id: id2, x_coord: x2, y_coord: y2}
        )
    ))
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
        assert_matches!(control_point_pair("c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0 trailing text"), Err(_));
        assert_matches!(control_point_pair( " n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0"), Err(_));
        assert_matches!(control_point_pair("c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364"   ), Err(_));
        assert_matches!(control_point_pair("c n0 n1 x568.542826048136 y117.691966641595 x54.4570607766205 y98.7300002744364 t0"), Err(_));
    }

}