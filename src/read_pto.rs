use std::str::FromStr;
use nom::{
    IResult,
    bytes::complete::{tag, take_while1},
    combinator::{map_res, eof},
    number::complete::double
};


//c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0
#[derive(Debug)]
struct ControlPoint {
    image_id: u64,
    x_coord: f64,
    y_coord: f64,
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






    #[test]
    fn f1 () {

        let (s,(cp1,cp2)) = control_point_pair("c n0 N1 x568.542826048136 y117.691966641595 X54.4570607766205 Y98.7300002744364 t0").unwrap();
        println!("{:?}", s);
        println!("{:?}", cp1);
        println!("{:?}", cp2);

    }
}