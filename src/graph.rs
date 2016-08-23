use std::f64;

use BitMap;
use line;
use Axis;

const W_ARROW: usize = 4;      //width of arrow
const W_NUMBER: usize = 4;     //number width in pixel
const H_NUMBER: usize = 5;     //number height in pixels
const W_BORDER: usize = 1;     //space around graph width
const H_ARROW_HALF: usize = 3;

const LEFT_SHIFT: usize = W_BORDER + W_NUMBER + H_NUMBER;
const RIGHT_SHIFT: usize = W_ARROW;


quick_error! {
    #[derive(Debug)]
    pub enum GraphError {
        NotEnoughPoints {
            description("There are not enough points to display on graph.")
        }
        NotEnoughSpace {
            description("There are not enough width and height to form graph with axis.")
        }
        NonUniquePoints {
            description("There are only one unique point. Can't construct line.")
        }
    }
}

pub type GraphResult = Result<Vec<u8>, GraphError>;


#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl<'a> From<&'a (f64, f64)> for Point {
    fn from(t: &'a (f64, f64)) -> Point {
        Point { x: t.0, y: t.1 }
    }
}

impl From<(f64, f64)> for Point {
    fn from(t: (f64, f64)) -> Point {
        Point { x: t.0, y: t.1 }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct DisplayPoint {
    pub x: usize,
    pub y: usize,
}


#[derive(Debug)]
pub struct Serie<'a, T, P>
    where T: Iterator<Item = P> + Clone,
          P: Into<Point> + PartialEq
{
    pub iter: T,
    color: &'a str,
    max_x: f64,
    max_y: f64,
    min_x: f64,
    min_y: f64,
}

impl<'a, T, P> Serie<'a, T, P>
    where T: Iterator<Item = P> + Clone,
          P: Into<Point> + PartialEq
{
    pub fn new(iter: T, color: &'a str) -> Result<Self, GraphError> {

        if iter.clone().nth(1).is_none() {
            return Err(GraphError::NotEnoughPoints);
        }

        let first = iter.clone().nth(0).unwrap();
        if !iter.clone().skip(1).any(move |p| p != first) {
            return Err(GraphError::NonUniquePoints);
        }

        let (max_x, min_x, max_y, min_y) = Self::calculate_max_min(iter.clone());

        Ok(Serie {
            iter: iter,
            color: color,
            max_x: max_x,
            max_y: max_y,
            min_x: min_x,
            min_y: min_y,
        })
    }

    fn calculate_max_min(iter: T) -> (f64, f64, f64, f64) {
        let (mut min_x, mut max_x) = (f64::INFINITY, f64::NEG_INFINITY);
        let (mut min_y, mut max_y) = (f64::INFINITY, f64::NEG_INFINITY);

        for p in iter {
            let p = p.into();
            if p.x > max_x {
                max_x = p.x;
            }
            if p.x < min_x {
                min_x = p.x;
            }
            if p.y > max_y {
                max_y = p.y;
            }
            if p.y < min_y {
                min_y = p.y;
            }
        }
        (max_x, min_x, max_y, min_y)
    }
}


#[derive(Debug)]
pub struct Chart {
    width: usize,
    height: usize,
    background_color: u8,
    axis_color: u8,
    pixs: Vec<u8>,
    picture: BitMap,
    axis_x: Option<Axis>,
    axis_y: Option<Axis>,
}

impl Chart {
    pub fn new(width: usize,
               height: usize,
               background_color: &str,
               axis_color: &str)
               -> Result<Self, GraphError> {

        if width < (2 * H_NUMBER + 2 * W_NUMBER + W_ARROW + 2 * W_BORDER) ||
           height < (2 * H_NUMBER + 2 * W_NUMBER + W_ARROW + 2 * W_BORDER) {
            return Err(GraphError::NotEnoughSpace);
        };

        let mut picture = BitMap::new(width, height);

        let background_color_number = picture.add_color(background_color);

        let axis_color_number = picture.add_color(axis_color);

        let size = width * height;

        let pixs = vec![background_color_number;  size];

        Ok(Chart {
            width: width,
            height: height,
            background_color: background_color_number,
            axis_color: axis_color_number,
            pixs: pixs,
            picture: picture,
            axis_x: None,
            axis_y: None,
        })
    }

    fn draw_axes<'a, T, P>(&mut self, serie: &Serie<'a, T, P>)
        where T: Iterator<Item = P> + Clone,
              P: Into<Point> + PartialEq
    {
        let axis_x = Axis::calculate_axis(serie.max_x, serie.min_x, self.width);

        let axis_y = Axis::calculate_axis(serie.max_y, serie.min_y, self.height).rotate();

        let minor_net = self.get_minor_net(&axis_x, &axis_y);

        let axis_color = self.axis_color;

        self.draw_pixels(axis_x.create_points(), axis_color);

        self.draw_pixels(axis_y.create_points(), axis_color);

        self.draw_pixels(minor_net, axis_color);

        self.axis_x = Some(axis_x);

        self.axis_y = Some(axis_y);
    }

    fn get_minor_net(&self, axis_x: &Axis, axis_y: &Axis) -> Vec<DisplayPoint> {
        let mut v: Vec<DisplayPoint> = vec![];
        for i in 0..axis_x.k_i {
            let shift = LEFT_SHIFT + ((axis_x.c_i * (i as f64)).round() as usize);
            for j in LEFT_SHIFT..(self.height - H_ARROW_HALF) {
                if j % 2 != 0 {
                    v.push(DisplayPoint { x: shift, y: j });
                }
            }
        }

        for i in 0..axis_y.k_i {
            let shift = LEFT_SHIFT + ((axis_y.c_i * (i as f64)).round() as usize);
            for j in LEFT_SHIFT..(self.width - H_ARROW_HALF) {
                if j % 2 != 0 {
                    v.push(DisplayPoint { x: j, y: shift });
                }
            }
        }
        v
    }




    pub fn create_bmp_vec<'a, T, P>(&mut self, serie: Serie<'a, T, P>) -> GraphResult
        where T: Iterator<Item = P> + Clone,
              P: Into<Point> + PartialEq
    {

        self.draw_axes(&serie);

        let func_points = {

            let function = self.serie_to_points(&serie);

            line::extrapolate(function).collect::<Vec<DisplayPoint>>()

        };

        let points_color_number = self.picture.add_color(serie.color);

        self.draw_pixels(func_points, points_color_number);

        self.picture.add_pixels(&self.pixs);

        Ok(self.picture.to_vec())
    }

    fn serie_to_points<'a, T, P>(&'a mut self,
                                 serie: &'a Serie<'a, T, P>)
                                 -> Box<Iterator<Item = DisplayPoint> + 'a>
        where T: Iterator<Item = P> + Clone,
              P: Into<Point> + PartialEq
    {
        let width_available = self.width - LEFT_SHIFT - RIGHT_SHIFT;

        let height_available = self.height - LEFT_SHIFT - RIGHT_SHIFT;

        let axis_x = self.axis_x.clone().unwrap();

        let axis_y = self.axis_y.clone().unwrap();

        let resolution_x: f64 = (axis_x.max_value - axis_x.min_value) / (width_available as f64);
        let resolution_y: f64 = (axis_y.max_value - axis_y.min_value) / (height_available as f64);

        let serie_iter = serie.iter.clone();

        Box::new(serie_iter.map(move |p| {
            let p = p.into();
            let mut id_x = ((p.x - axis_x.min_value) / resolution_x).round() as usize;
            let mut id_y = ((p.y - axis_y.min_value) / resolution_y).round() as usize;

            if id_x == self.width {
                id_x -= 1;
            }
            if id_y == self.height {
                id_y -= 1;
            }
            DisplayPoint {
                x: (id_x + LEFT_SHIFT),
                y: (id_y + LEFT_SHIFT),
            }
        }))

    }


    fn draw_pixels(&mut self, points: Vec<DisplayPoint>, color: u8) {
        for p in points {
            let i = p.y * self.width + p.x;
            self.pixs[i] = color;
        }
    }
}

#[test]
fn not_enough_space_test() {
    let result = Chart::new(10, 15, "#ffffff", "#000000");
    assert_eq!(result.unwrap_err().to_string(),
               "There are not enough width and height to form graph with axis.");
}

#[test]
fn not_enough_points_test() {
    let v: Vec<(f64, f64)> = vec![];
    let result = Serie::new(v.into_iter(), "#0000ff");
    assert_eq!(result.unwrap_err().to_string(),
               "There are not enough points to display on graph.");
}

#[test]
fn one_point_test() {
    let p = vec![(1f64, 1f64)];
    let result = Serie::new(p.into_iter(), "#0000ff");
    assert_eq!(result.unwrap_err().to_string(),
               "There are not enough points to display on graph.");
}

#[test]
fn two_identical_point_test() {
    let p = vec![(1f64, 1f64), (1f64, 1f64)];
    let result = Serie::new(p.into_iter(), "#0000ff");
    assert_eq!(result.unwrap_err().to_string(),
               "There are only one unique point. Can't construct line.");
}

#[test]
fn can_create_array() {
    let p = vec![(1f64, 1f64), (2f64, 2f64), (3f64, 3f64)];
    let serie = Serie::new(p.into_iter(), "#0000ff").unwrap();
    let mut chart = Chart::new(100, 100, "#ffffff", "#000000").unwrap();
    let bmp = chart.create_bmp_vec(serie).unwrap();
    for p in bmp {
        println!("{}", p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn create_graph_2_points(b: &mut Bencher) {
        b.iter(|| {
            let p = vec![(1f64, 1f64), (2f64, 2f64), (3f64, 3f64)];
            let serie = Serie::new(p.into_iter(), "#0000ff").unwrap();
            let mut chart = Chart::new(740, 480, "#ffffff", "#000000").unwrap();
            let _ = chart.create_bmp_vec(serie).unwrap();
        })
    }

    #[bench]
    fn create_graph_1000_points(b: &mut Bencher) {
        b.iter(|| {
            let p: Vec<_> = formula!(y(x): f64 = {x*x}, x = [0f64, 1000f64; 1f64]).collect();
            let serie = Serie::new(p.into_iter(), "#0000ff").unwrap();
            let mut chart = Chart::new(740, 480, "#ffffff", "#000000").unwrap();
            let _ = chart.create_bmp_vec(serie).unwrap();
        })
    }

    #[bench]
    #[ignore]
    fn create_graph_1000000_points(b: &mut Bencher) {
        b.iter(|| {
            let p: Vec<_> = formula!(y(x): f64 = {x*x}, x = [0f64, 1000f64; 0.001f64]).collect();
            let serie = Serie::new(p.into_iter(), "#0000ff").unwrap();
            let mut chart = Chart::new(740, 480, "#ffffff", "#000000").unwrap();
            let _ = chart.create_bmp_vec(serie).unwrap();
        })
    }
}
