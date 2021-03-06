//! generate an SVG from the ascii text using the default settings
//!
//! ```
//! let input = r#"
//! .-------------------------------------.
//! | Hello here and there and everywhere |
//! '-------------------------------------'
//! "#;
//! println!("svg: {}",svgbob::to_svg(input));
//! ```
//! 
//! <svg font-family="Electrolize,Titillium Web, Trebuchet MS, Arial" font-size="14" height="80" width="344" xmlns="http://www.w3.org/2000/svg">
//! <defs>
//! <marker id="triangle" markerHeight="10" markerUnits="strokeWidth" markerWidth="10" orient="auto" refX="0" refY="5" viewBox="0 0 14 14">
//! <path d="M 0 0 L 10 5 L 0 10 z"/>
//! </marker>
//! </defs>
//! <style>
//!     line, path {
//!       stroke: black;
//!       stroke-width: 1;
//!     }
//! </style>
//! <path d=" M 36 28 L 36 48 M 40 24 A 4 4 0 0 0 36 28 M 40 24 L 336 24 M 340 28 L 340 48 M 340 28 A 4 4 0 0 0 336 24 M 36 32 L 36 48 M 340 32 L 340 48 M 36 48 L 36 52 A 4 4 0 0 0 40 56 L 336 56 M 340 48 L 340 52 M 336 56 A 4 4 0 0 0 340 52" fill="none"/>
//! <path d="" fill="none" stroke-dasharray="3 3"/>
//! <text x="50" y="44">
//! Hello here and there and everywhere
//! </text>
//! </svg>
//! 
//! 
#![deny(warnings)]
extern crate svg;
extern crate unicode_width;



use svg::Node;
use svg::node::element::Circle as SvgCircle;
use svg::node::element::Path as SvgPath;
use svg::node::element::Line as SvgLine;
use svg::node::element::Text as SvgText;
use svg::node::element::Style;
use svg::node::element::SVG;
use svg::node::element::Definitions;
use svg::node::element::Marker;
use optimizer::Optimizer;
use self::Feature::Arrow;
use self::Feature::Circle;
use self::Feature::Nothing;
use self::Stroke::Solid;
use self::Stroke::Dashed;
use unicode_width::UnicodeWidthStr;
use unicode_width::UnicodeWidthChar;

mod optimizer;


/// generate an SVG from the ascii text input
///
/// Usage:
/// 
/// ```
/// let input = "------->";
/// println!("svg: {}", svgbob::to_svg(input));
/// ``` 
/// 
/// commercial version enhances memes automatically
pub fn to_svg(input: &str) -> SVG {
    let settings = &Settings::default();
    Grid::from_str(&input).get_svg(settings)
}

pub fn to_svg_with_size(input: &str, text_width: f32, text_height: f32) -> SVG {
    let settings = &Settings::with_size(text_width, text_height);
    Grid::from_str(&input).get_svg(settings)
}

pub fn to_svg_with_size_nooptimization(input: &str, text_width: f32, text_height: f32) -> SVG {
    let mut settings = Settings::no_optimization();
    settings.text_width = text_width;
    settings.text_height = text_height;
    Grid::from_str(&input).get_svg(&settings)
}


pub struct Settings {
    text_width: f32,
    text_height: f32,
    /// do optimization? if false then every piece are disconnected
    optimize: bool,
    /// if optmization is enabled,
    /// true means all reduceable paths will be in 1 path definition
    compact_path: bool,
}

impl Settings {

    pub fn with_size(text_width: f32, text_height: f32) -> Self{
         Settings{
            text_width: text_width,
            text_height: text_height,
            optimize: true,
            compact_path: true,
         }
    }
    pub fn no_optimization() -> Settings {
        let mut settings = Settings::default();
        settings.optimize = false;
        settings.compact_path = false;
        settings
    }

    pub fn separate_lines() -> Settings {
        let mut settings = Settings::default();
        settings.optimize = true;
        settings.compact_path = false;
        settings
    }

    pub fn compact() -> Settings {
        let mut settings = Settings::default();
        settings.optimize = true;
        settings.compact_path = true;
        settings
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            text_width: 8.0,
            text_height: 16.0,
            optimize: true,
            compact_path: true,
        }
    }
}

enum SvgElement {
    Circle(SvgCircle),
    Line(SvgLine),
    Path(SvgPath),
    Text(SvgText),
}


#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub enum Stroke {
    Solid,
    Dashed,
}


#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub enum Feature {
    Arrow, //end
    Circle, //start
    Nothing,
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Point {
    x: f32,
    y: f32,
}
impl Point {
    fn new(x: f32, y: f32) -> Point {
        Point { x: x, y: y }
    }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct Loc {
    x: isize,
    y: isize,
}

impl Loc {
    fn new(x: isize, y: isize) -> Loc {
        Loc { x: x, y: y }
    }

    pub fn top(&self) -> Loc {
        Loc {
            x: self.x,
            y: self.y - 1,
        }
    }
    pub fn left(&self) -> Loc {
        Loc {
            x: self.x - 1,
            y: self.y,
        }
    }
    pub fn bottom(&self) -> Loc {
        Loc {
            x: self.x,
            y: self.y + 1,
        }
    }
    pub fn right(&self) -> Loc {
        Loc {
            x: self.x + 1,
            y: self.y,
        }
    }

    pub fn top_left(&self) -> Loc {
        Loc {
            x: self.x - 1,
            y: self.y - 1,
        }
    }

    pub fn top_right(&self) -> Loc {
        Loc {
            x: self.x + 1,
            y: self.y - 1,
        }
    }

    pub fn bottom_left(&self) -> Loc {
        Loc {
            x: self.x - 1,
            y: self.y + 1,
        }
    }

    pub fn bottom_right(&self) -> Loc {
        Loc {
            x: self.x + 1,
            y: self.y + 1,
        }
    }

    /// get the 8 neighbors
    pub fn neighbors(&self) -> Vec<Loc> {
        vec![self.top(), 
             self.bottom(),
             self.left(),
             self.right(),
             self.top_left(),
             self.top_right(),
             self.bottom_left(),
             self.bottom_right(),
            ]
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub enum Element {
    Circle(Point, f32, String),
    Line(Point, Point, Stroke, Feature),
    Arc(Point, Point, f32, bool),
    Text(Loc, String),
    Path(Point, Point, String, Stroke),
}

impl Element {
    fn solid_circle(c: &Point, r: f32) -> Element{
        Element::Circle(c.clone(), r, "solid".into())
    }
    fn open_circle(c: &Point, r: f32) -> Element{
        Element::Circle(c.clone(), r, "open".into())
    }
    fn solid_line(s: &Point, e: &Point) -> Element {
        Element::line(s, e, Solid, Nothing)
    }

    fn line(s: &Point, e: &Point, stroke: Stroke, feature: Feature) -> Element {
        Element::Line(s.clone(), e.clone(), stroke, feature)
    }
    fn arc(s: &Point, e: &Point, radius: f32, sweep: bool) -> Element {
        Element::Arc(s.clone(), e.clone(), radius, sweep)
    }
    // this path can chain to the other path
    // chain means the paths can be arranged and express in path definition
    // if self.end == path.start
    /*
    fn chain(&self, other: &Element) -> Option<Vec<Element>> {
        match *self {
            Element::Line(_, ref e, ref stroke, ref feature) => {
                match *other {
                    Element::Line(ref s2, _, ref stroke2, ref feature2) => {
                        if e == s2 && stroke == stroke2 //must have same stroke
                       && *feature != Arrow // no arrow on the first
                       && *feature2 != Circle // no start marker on the second
                        {
                            Some(vec![self.clone(), other.clone()])
                        } else {
                            None
                        }
                    }
                    Element::Arc(ref s2, _, _, _) => {
                        if e == s2 && *feature != Arrow {
                            Some(vec![self.clone(), other.clone()])
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            Element::Arc(_, ref e, _, _) => {
                match *other {
                    Element::Line(ref s2, _, ref stroke2, _) => {
                        match *stroke2 {
                            Solid => {
                                // arcs are always solid, so match only solid line to arc
                                if e == s2 {
                                    Some(vec![self.clone(), other.clone()])
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }
                    Element::Arc(ref s2, _, _, _) => {
                        if e == s2 {
                            Some(vec![self.clone(), other.clone()])
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            Element::Text(_, _) => {
                // text can reduce, but not chain
                None
            }
            Element::Path(_, _, _, _) => None,
        }
    }
    */

    // if this element can reduce the other, return the new reduced element
    // for line it has to be collinear and in can connect start->end->start
    // for text, the other text should apear on the right side of this text
    fn reduce(&self, other: &Element) -> Option<Element> {
        match *self {
            Element::Line(ref s, ref e, ref stroke, ref feature) => {
                match *other {
                    Element::Line(ref s2, ref e2, ref stroke2, ref feature2) => {
                        // note: dual 3 point check for trully collinear lines
                        if collinear(s, e, s2) && collinear(s, e, e2) && e == s2 &&
                           stroke == stroke2 && *feature == Nothing
                           && *feature2 != Circle
                           {
                            let reduced = Some(Element::Line(s.clone(),
                                                             e2.clone(),
                                                             stroke.clone(),
                                                             feature2.clone()));
                            reduced
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            Element::Text(ref loc, ref text) => {
                match *other {
                    Element::Text(ref loc2, ref text2) => {
                        // reduce if other is next to it
                        let uwidth = text.width() as isize;
                        if loc.y == loc2.y && loc.x + uwidth == loc2.x {
                            let merged_text = text.clone() + text2;
                            let reduced = Some(Element::Text(loc.clone(), merged_text));
                            reduced
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }


    fn to_svg(&self, settings: &Settings) -> SvgElement {
        match *self {
            Element::Circle(ref c, r, ref class) => {
                let svg_circle = SvgCircle::new()
                    .set("class",class.clone())
                    .set("cx", c.x)
                    .set("cy", c.y)
                    .set("r", r);

                SvgElement::Circle(svg_circle)
            },
            Element::Line(ref s, ref e, ref stroke, ref feature) => {
                let mut svg_line = SvgLine::new()
                    .set("x1", s.x)
                    .set("y1", s.y)
                    .set("x2", e.x)
                    .set("y2", e.y);

                match *feature {
                    Arrow => {
                        svg_line.assign("marker-end", "url(#triangle)");
                    },
                    Circle => {
                        svg_line.assign("marker-start", "url(#circle)");
                    },
                    Nothing => (),
                };
                match *stroke {
                    Solid => (),
                    Dashed => {
                        svg_line.assign("stroke-dasharray", (3, 3));
                        svg_line.assign("fill", "none");
                    }
                };

                SvgElement::Line(svg_line)
            }
            Element::Arc(ref s, ref e, radius, sweep) => {
                let sweept = if sweep { "1" } else { "0" };
                let d = format!("M {} {} A {} {} 0 0 {} {} {}",
                                s.x,
                                s.y,
                                radius,
                                radius,
                                sweept,
                                e.x,
                                e.y);
                let svg_arc = SvgPath::new()
                    .set("d", d)
                    .set("fill", "none");
                SvgElement::Path(svg_arc)
            }
            Element::Text(ref loc, ref string) => {
                let sx = loc.x as f32 * settings.text_width + settings.text_width / 4.0;
                let sy = loc.y as f32 * settings.text_height + settings.text_height * 3.0 / 4.0;
                let mut svg_text = SvgText::new()
                    .set("x", sx)
                    .set("y", sy);
                let text_node = svg::node::Text::new(string.to_string());
                svg_text.append(text_node);
                SvgElement::Text(svg_text)
            }
            Element::Path(_, _, ref d, ref stroke) => {
                let mut path = SvgPath::new()
                    .set("d", d.to_owned())
                    .set("fill", "none");

                match *stroke {
                    Solid => (),
                    Dashed => {
                        path.assign("stroke-dasharray", (3, 3));
                    }
                };
                SvgElement::Path(path)
            }
        }
    }
}


// 3 points are collinear when the area of the triangle connecting them is 0;
fn collinear(a: &Point, b: &Point, c: &Point) -> bool {
    a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y) == 0.0
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct GChar {
    /// the characters in this Element
    string: String,
    /// total width of all characters in chars
    width: usize,
}

impl GChar{
    fn new(ch:char) -> Self {
        let mut s = String::new();
        s.push(ch);
        let width =UnicodeWidthStr::width(&*s);
        GChar{
            string: s,
            width: width
        }
    }

    fn from_str(s:&str) -> Self{
        let width =UnicodeWidthStr::width(s);
        GChar{
            string: s.into(),
            width: width
        }
    }
    
    fn push_str(self, s: &str) -> Self {
        let mut string = self.string.clone();
        string.push_str(s);
        GChar::from_str(&string)
    }
}


#[derive(Debug)]
pub struct Grid {
    //source: String,
    rows: usize,
    columns: usize,
    lines: Vec<Vec<GChar>>,
}
impl Grid {
    /// instantiate a grid from input ascii textinstantiate a grid from input ascii text
    pub fn from_str(s: &str) -> Grid {
        let lines: Vec<&str> = s.lines().collect();
        let mut line_gchars = Vec::with_capacity(lines.len());
        
        for line in lines{
            let mut gchars = Vec::with_capacity(line.len());
            let mut zero_ch = None;
            for ch in line.chars(){
                if let Some(unicode_width) = ch.width(){
                    // if width is zero add the char to previous buffer
                    if unicode_width == 0 {
                        zero_ch = Some(ch);
                    }else{
                        match zero_ch{
                            Some(prefend_ch) => {
                                let mut s = String::new();
                                s.push(prefend_ch);
                                s.push(ch);
                                let last_gchar:Option<GChar> = gchars.pop();
                                if let Some(last_gchar) = last_gchar{
                                    let gchar = last_gchar.push_str(&s);
                                    gchars.push(gchar);
                                }
                                zero_ch = None;
                            }
                            None => {
                                let gchar = GChar::new(ch);
                                gchars.push(gchar);
                            }
                        }
                    }
                }
            } 
            line_gchars.push(gchars);
        }
        let mut max = 0;
        for lg in &line_gchars{
            let mut line_width = 0;
            for gchar in lg{
                line_width += gchar.width; 
            } 
            if line_width >= max{
                max = line_width;
            }
        }

        Grid {
            rows: line_gchars.len(),
            columns: max,
            lines: line_gchars,
        }
    }

    fn get(&self, loc: &Loc) -> Option<&GChar> {
        match self.lines.get(loc.y as usize) {
            Some(line) => {
                let mut total_width = 0;
                for gchar in line{
                    if total_width == loc.x{
                        return Some(gchar)
                    }
                    total_width += gchar.width as isize;
                }
                None
            }
            None => None,
        }
    }


    fn is_char<F>(&self, loc: &Loc, f: F) -> bool
        where F: Fn(&str) -> bool
    {
        if let Some(gchar) = self.get(loc){
             f(&gchar.string)
        }else{
            false
        }
    }


    



    /// get the elements on this location
    /// variable names:
    /// the grid is 8x8 divided into 4 equal parts at each vertical and horizontal dimension.
    /// a,b,c,d,e  is start,quater,center,3quarters, end respectively
    ///
    /// combining [a,b,c,d,e] * h]
    /// ah,bh,ch,dh,eh are horizontal increments derived from dividing the textwidth into 4 equal parts.
    ///
    /// combining [a,b,c,d,e] * [v]
    /// av,bv,cv,dv,ev are vertical increments derived from diving the textheight into 4 equal parts
    ///
    /// combining [a,b,c,d,e] * [x] and [a,b,c,d,e] * [y]
    /// and you will get the location of the points in the grid that describe the relative location
    /// of the point from the starting location of the elements
    /// all intersection and junction points fall exactly to any of the grid points
    ///
    fn get_elements(&self, x: isize, y: isize, settings: &Settings) -> Option<Vec<Element>> {
        let text_width = settings.text_width;
        let text_height = settings.text_height;
        let measurex = x as f32 * text_width;
        let measurey = y as f32 * text_height;
        let arc_radius = text_width / 2.0;

        // horizontal divide
        let ah = 0.0;
        let bh = text_width / 4.0;
        let ch = text_width / 2.0;
        let dh = text_width * 3.0 / 4.0;
        let eh = text_width;

        // vertical divide
        let av = 0.0;
        let bv = text_height / 4.0;
        let cv = text_height / 2.0;
        let dv = text_height * 3.0 / 4.0;
        let ev = text_height;

        let ax = measurex + ah;
        let ay = measurey + av;
        let bx = measurex + bh;
        let by = measurey + bv;
        let cx = measurex + ch;
        let cy = measurey + cv;
        let dx = measurex + dh;
        let dy = measurey + dv;
        let ex = measurex + eh;
        let ey = measurey + ev;


        // point locations
        let center_top = &Point::new(cx, ay);
        let center_bottom = &Point::new(cx, ey);
        let mid_left = &Point::new(ax, cy);
        let mid_right = &Point::new(ex, cy);
        let high_left = &Point::new(ax, ay);
        let high_right = &Point::new(ex, ay);
        let low_left = &Point::new(ax, ey);
        let low_right = &Point::new(ex, ey);

        // point 5x5 locations
        let axay = &Point::new(ax, ay);
        let bxby = &Point::new(bx, by);
        let cxcy = &Point::new(cx, cy);
        let dxdy = &Point::new(dx, dy);
        let exey = &Point::new(ex, ey);

        let axcy = &Point::new(ax, cy);
        let bxdy = &Point::new(bx, dy);
        let bxcy = &Point::new(bx, cy);
        let cxay = &Point::new(cx, ay);
        let cxey = &Point::new(cx, ey);
        let cxdy = &Point::new(cx, dy);
        let cxby = &Point::new(cx, by);
        let dxby = &Point::new(dx, by);
        let dxcy = &Point::new(dx, cy);
        let excy = &Point::new(ex, cy);
        let dxey = &Point::new(dx, ey);
        let dxay = &Point::new(dx, ay);
        let bxay = &Point::new(bx, ay);
        let bxey = &Point::new(bx, ey);
        let axey = &Point::new(ax, ey);
        let exay = &Point::new(ex, ay);

        // extended points
        let axbhey = &Point::new(ax - bh, ey);
        let exbhey = &Point::new(ex + bh, ey);
        let axbhay = &Point::new(ax - bh, ay);
        let exbhay = &Point::new(ex + bh, ay);
        let axchay = &Point::new(ax - ch, ay);
        let exchey = &Point::new(ex + ch, ey);
        let exchay = &Point::new(ex + ch, ay);
        let axchey = &Point::new(ax - ch, ey);
        let axehey = &Point::new(ax - eh, ey);
        let axehay = &Point::new(ax - eh, ay);
        let exdhey = &Point::new(ex + dh, ey);
        let exehay = &Point::new(ex + eh, ay);
        let axdhey = &Point::new(ax - dh, ey);
        let exehey = &Point::new(ex + eh, ey);
        let axdhay = &Point::new(ax - dh, ay);
        let exdhay = &Point::new(ex + dh, ay);
        let exchcy = &Point::new(ex + ch, cy);
        let axchcy = &Point::new(ax - ch, cy);
        let exchby = &Point::new(ex + ch, by);
        let cxeybv = &Point::new(cx, ey + bv);
        let cxeycv = &Point::new(cx, ey + cv);
        let exchdy = &Point::new(ex + ch, dy);
        let cxaybv = &Point::new(cx, ay - bv);
        let axchby = &Point::new(ax - ch, by);
        let axchdy = &Point::new(ax - ch, dy);
        let cxaycv = &Point::new(cx, ay - cv);
        let excheycv = &Point::new(ex + ch, ey + cv);
        let axcheycv = &Point::new(ax - ch, ey + cv);
        let axchaycv = &Point::new(ax - ch, ay - cv);
        let exchaycv = &Point::new(ex + ch, ay - cv);


        // grid lines
        let axay_bxby = Element::solid_line(axay, bxby);
        let cxcy_axcy = Element::solid_line(cxcy, axcy);
        let cxcy_cxay = Element::solid_line(cxcy, cxay);
        let cxcy_cxey = Element::solid_line(cxcy, cxey);
        let cxcy_excy = Element::solid_line(cxcy, excy);

        let cxdy_cxey = Element::solid_line(cxdy, cxey);
        let cxay_cxby = Element::solid_line(cxay, cxby);
        let dxby_exay = Element::solid_line(dxby, exay);
        let axey_bxdy = Element::solid_line(axey, bxdy);
        let exey_dxdy = Element::solid_line(exey, dxdy);
        let dxcy_excy = Element::solid_line(dxcy, excy);
        let bxcy_axcy = Element::solid_line(bxcy, axcy);
        let exay_dxby = Element::solid_line(exay, dxby);
        let cxey_cxdy = Element::solid_line(cxey, cxdy);
        let dxdy_exey = Element::solid_line(dxdy, exey);
        let cxcy_exey = Element::solid_line(cxcy, exey);
        let cxcy_axey = Element::solid_line(cxcy, axey);
        let axay_cxcy = Element::solid_line(axay, cxcy);
        let cxcy_exay = Element::solid_line(cxcy, exay);
        let cxay_cxcy = Element::solid_line(cxay, cxcy);
        let axay_excy = Element::solid_line(axay, excy);
        let axcy_exey = Element::solid_line(axcy, exey);
        let axcy_exay = Element::solid_line(axcy, exay);
        let axey_excy = Element::solid_line(axey, excy);
        let exay_axcy = Element::solid_line(exay, axcy);
        let excy_axey = Element::solid_line(excy, axey);
        let axey_cxcy = Element::solid_line(axey, cxcy);
        let cxey_cxcy = Element::solid_line(cxey, cxcy);
        let axay_cxby = Element::solid_line(axay, cxby);
        let cxby_exay = Element::solid_line(cxby, exay);
        let axcy_cxdy = Element::solid_line(axcy, cxdy);
        let cxdy_excy = Element::solid_line(cxdy, excy);
        let cxdy_exey = Element::solid_line(cxdy, exey);
        let cxdy_axcy = Element::solid_line(cxdy, axcy);
        let cxdy_axey = Element::solid_line(cxdy, axey);
        let cxby_axcy = Element::solid_line(cxby, axcy);
        let cxby_excy = Element::solid_line(cxby, excy);

        let axcy_exchby = Element::solid_line(axcy, exchby);
        let cxdy_cxeybv = Element::solid_line(cxdy, cxeybv);
        let cxaybv_cxby = Element::solid_line(cxaybv, cxby);

        let exay_axehey = Element::solid_line(exay, axehey);
        let axay_exehey = Element::solid_line(axay, exehey);
        let axcy_exchcy = Element::solid_line(axcy, exchcy);
        let axchey_exehey = Element::solid_line(axchey, exehey);
        let excy_axchcy = Element::solid_line(excy, axchcy);
        let cxdy_cxay = Element::solid_line(cxdy, cxay);
        let axcy_exchdy = Element::solid_line(axcy, exchdy);
        let axchby_excy = Element::solid_line(axchby, excy);
        let axchdy_excy = Element::solid_line(axchdy, excy);
        let cxay_cxeycv = Element::solid_line(cxay, cxeycv);
        let cxaycv_cxey = Element::solid_line(cxaycv, cxey);
        let axay_excheycv = Element::solid_line(axay, excheycv);
        let exay_axcheycv = Element::solid_line(exay, axcheycv);
        let exey_axchaycv = Element::solid_line(exey, axchaycv);
        let axey_exchaycv = Element::solid_line(axey, exchaycv);

        // common arc
        let arc_axcy_dxby = Element::arc(axcy, dxby, arc_radius * 2.0, false);
        let arc_bxby_excy = Element::arc(bxby, excy, arc_radius * 2.0, false);
        let arc_axcy_bxby = Element::arc(axcy, bxby, arc_radius, false);
        let arc_cxdy_axcy = Element::arc(cxdy, axcy, arc_radius, false);
        let arc_cxby_excy = Element::arc(cxby, excy, arc_radius, false);
        let arc_dxdy_axcy = Element::arc(dxdy, axcy, arc_radius * 2.0, false);
        let arc_excy_cxdy = Element::arc(excy, cxdy, arc_radius, false);
        let arc_excy_bxdy = Element::arc(excy, bxdy, arc_radius * 2.0, false);
        let arc_dxby_excy = Element::arc(dxby, excy, arc_radius, false);
        let arc_bxdy_axcy = Element::arc(bxdy, axcy, arc_radius, false);
        let arc_excy_dxdy = Element::arc(excy, dxdy, arc_radius, false);
        let arc_dxcy_bxdy = Element::arc(dxcy, bxdy, arc_radius * 2.0, false);
        let arc_dxdy_bxcy = Element::arc(dxdy, bxcy, arc_radius * 2.0, false);
        let arc_bxby_dxcy = Element::arc(bxby, dxcy, arc_radius * 2.0, false);
        let arc_bxdy_bxby = Element::arc(bxdy, bxby, arc_radius * 2.0, false);
        let arc_dxby_dxdy = Element::arc(dxby, dxdy, arc_radius * 2.0, false);
        let arc_dxby_cxdy = Element::arc(dxby, cxdy, arc_radius * 4.0, false);
        let arc_bxdy_cxby = Element::arc(bxdy, cxby, arc_radius * 4.0, false);
        let arc_cxby_dxdy = Element::arc(cxby, dxdy, arc_radius * 4.0, false);
        let arc_cxdy_bxby = Element::arc(cxdy, bxby, arc_radius * 4.0, false);
        let arc_dxay_dxey = Element::arc(dxay, dxey, arc_radius * 4.0, false);
        let arc_bxey_bxay = Element::arc(bxey, bxay, arc_radius * 4.0, false);
        let arc_excy_axcy = Element::arc(excy, axcy, arc_radius * 4.0, false);
        let arc_axcy_excy = Element::arc(axcy, excy, arc_radius * 4.0, false);
        let arc_bxcy_dxby = Element::arc(bxcy, dxby, arc_radius * 4.0, false);
        let arc_axcy_axay = Element::arc(axcy, axay, arc_radius * 4.0, false);
        let arc_axey_exey = Element::arc(axey, exey, arc_radius * 4.0, false);
        let arc_cxay_exey = Element::arc(cxay, exey, arc_radius * 8.0, false);
        let arc_exay_cxey = Element::arc(exay, cxey, arc_radius * 8.0, false);
        let arc_cxey_axay = Element::arc(cxey, axay, arc_radius * 8.0, false);
        let arc_axey_cxay = Element::arc(axey, cxay, arc_radius * 8.0, false);
        let arc_cxey_axcy = Element::arc(cxey, axcy, arc_radius * 2.0, false);
        let arc_axcy_cxay = Element::arc(axcy, cxay, arc_radius * 2.0, false);
        let arc_excy_cxey = Element::arc(excy, cxey, arc_radius * 2.0, false);
        let arc_cxay_excy = Element::arc(cxay, excy, arc_radius * 2.0, false);

        // extended arc
        let arc_exdhey_axehay = Element::arc(exdhey, axehay, arc_radius * 10.0, false);
        let arc_exchey_dxay = Element::arc(exchey, dxay, arc_radius * 10.0, false);
        let arc_exehay_axdhey = Element::arc(exehay, axdhey, arc_radius * 10.0, false);
        let arc_bxay_axchey = Element::arc(bxay, axchey, arc_radius * 10.0, false);
        let arc_axdhay_exehey = Element::arc(axdhay, exehey, arc_radius * 10.0, false);
        let arc_axchay_bxey = Element::arc(axchay, bxey, arc_radius * 10.0, false);
        let arc_axehey_exdhay = Element::arc(axehey, exdhay, arc_radius * 10.0, false);
        let arc_dxey_exchay = Element::arc(dxey, exchay, arc_radius * 10.0, false);
        let arc_axey_cxdy = Element::arc(axey, cxdy, arc_radius, false);
        let arc_cxdy_exey = Element::arc(cxdy, exey, arc_radius, false);
        let arc_excy_axbhey = Element::arc(excy, axbhey, arc_radius * 4.0, false);
        let arc_exbhey_axcy = Element::arc(exbhey, axcy, arc_radius * 4.0, false);
        let arc_axbhay_excy = Element::arc(axbhay, excy, arc_radius * 4.0, false);
        let arc_axcy_exbhay = Element::arc(axcy, exbhay, arc_radius * 4.0, false);
        let arc_axcy_cxby = Element::arc(axcy, cxby, arc_radius, false);

        // common path lines
        let vertical = Element::solid_line(center_top, center_bottom);
        let horizontal = Element::solid_line(mid_left, mid_right);
        let slant_left = Element::solid_line(high_left, low_right);
        let slant_right = Element::solid_line(low_left, high_right);
        let low_horizontal = Element::solid_line(low_left, low_right);

        // extended lines
        let low_horizontal_extend_left_half = Element::solid_line(low_right,
                                                                  &Point::new(ax - ch, ey));
        let low_horizontal_extend_right_half = Element::solid_line(low_left,
                                                                   &Point::new(ex + ch, ey));
        let low_horizontal_extend_left_full = Element::solid_line(low_right,
                                                                  &Point::new(ax - eh, ey));
        let low_horizontal_extend_right_full = Element::solid_line(low_left,
                                                                   &Point::new(ex + eh, ey));

        // dashed lines
        let vertical_dashed = Element::line(center_top, center_bottom, Dashed, Nothing);
        let horizontal_dashed = Element::line(mid_left, mid_right, Dashed, Nothing);
        let low_horizontal_dashed = Element::line(low_left, low_right, Dashed, Nothing);

        let arrow_down = Element::line(center_top,
                                       center_bottom,
                                       Solid,
                                       Arrow);
        let arrow_down_dashed = Element::line(center_top,
                                              center_bottom,
                                              Dashed,
                                              Arrow);
        let arrow_up = Element::line(center_bottom,
                                     center_top,
                                     Solid,
                                     Arrow);
        let arrow_up_dashed = Element::line(center_bottom,
                                            center_top,
                                            Dashed,
                                            Arrow);
        let arrow_left = Element::line(mid_right, cxcy, Solid, Arrow);
        let arrow_left_dashed =
            Element::line(mid_right, cxcy, Dashed, Arrow);
        let arrow_right = Element::line(mid_left, cxcy, Solid, Arrow);
        let arrow_right_dashed =
            Element::line(mid_left, cxcy, Dashed, Arrow);
        let arrow_bottom_left =
            Element::line(high_right, cxcy, Solid, Arrow);
        let arrow_bottom_right =
            Element::line(high_left, cxcy, Solid, Arrow);
        let arrow_top_left = Element::line(low_right, cxcy, Solid, Arrow);
        let arrow_top_right = Element::line(low_left, cxcy, Solid, Arrow);

        let junction_circle = Element::solid_circle(cxcy,ch); 
        let open_junction = Element::open_circle(cxcy,ch); 

        // relative location of characters
        let this = &Loc::new(x, y);
        let top = &this.top();
        let left = &this.left();
        let bottom = &this.bottom();
        let right = &this.right();
        let top_left = &this.top_left();
        let top_right = &this.top_right();
        let bottom_left = &this.bottom_left();
        let bottom_right = &this.bottom_right();

        // left of left
        let left_left = &this.left().left();
        let right_right = &this.right().right();

        let connects_left = self.is_char(left, is_horizontal);
        let connects_right = self.is_char(right, is_horizontal);
        let connects_top = self.is_char(top, is_vertical);
        let connects_bottom = self.is_char(bottom, is_vertical);
        let connects_top_left = self.is_char(top_left, is_slant_left);
        let connects_top_right = self.is_char(top_right, is_slant_right);
        let connects_bottom_left = self.is_char(bottom_left, is_slant_right);
        let connects_bottom_right = self.is_char(bottom_right, is_slant_left);
        let connects_major4 = connects_left || connects_right || connects_top || connects_bottom;
        let connects_aux4 = connects_top_left || connects_top_right || connects_bottom_left || connects_bottom_right;
        let connects = connects_major4 || connects_aux4;

        // FIXME: need more exhaustive list, for use case that makes sense matching
        let match_list: Vec<(bool, Vec<Element>)> = 
            vec![
                /*

                 \|/
                 -*- (asterisk)
                 /|\

                */
                (self.is_char(this, is_asterisk) && connects,
                 vec![junction_circle.clone()]
                ),
                /*
                  
                \|/
                -o-
                /|\

                */
                (self.is_char(this, is_o) && connects,
                 vec![open_junction.clone()]
                ),
                /*
                    |
                */
                (self.is_char(this, is_vertical),
                 vec![vertical.clone()]
                ),
                /*
                    -
                */
                (self.is_char(this, is_horizontal),
                 vec![horizontal.clone()]
                ),

                /*
                    _
                */
                (self.is_char(this, is_low_horizontal),
                 vec![low_horizontal.clone()]
                ),
                /*
                   :
                   :
                   must have at least 1 align to it to be treated as vertical 
                */
                (self.is_char(this, is_vertical_dashed)
                  && (self.is_char(top, is_vertical_dashed)
                     || self.is_char(bottom, is_vertical_dashed)),
                  vec![vertical_dashed.clone()]
                ),

                /*
                   ==  at least 2 next to it
                */
                (self.is_char(this, is_horizontal_dashed)
                  && ((self.is_char(left, is_horizontal_dashed)
                     && self.is_char(right, is_horizontal_dashed)
                     )
                     ||
                     (self.is_char(left, is_horizontal_dashed)
                     && self.is_char(left_left, is_horizontal_dashed)
                     )
                     ||
                     (self.is_char(right, is_horizontal_dashed)
                      && self.is_char(right_right, is_horizontal_dashed)
                     )
                     ),
                  vec![horizontal_dashed.clone()]
                ),
                /*
                   ...  at least 2 next to it
                */
                (self.is_char(this, is_low_horizontal_dashed)
                  && ((self.is_char(left, is_low_horizontal_dashed) //left & right
                      && self.is_char(right, is_low_horizontal_dashed)
                      )
                     || 
                      (self.is_char(left, is_low_horizontal_dashed)
                       && self.is_char(left_left, is_low_horizontal_dashed)
                      )
                     || 
                      (self.is_char(right, is_low_horizontal_dashed)
                       && self.is_char(right_right, is_low_horizontal_dashed)
                      )
                     ),
                  vec![low_horizontal_dashed.clone()]
                ),
                /*
                    /
                */
                (self.is_char(this, is_slant_right),
                 vec![slant_right.clone()]
                ),
                /*
                    \
                */
                (self.is_char(this, is_slant_left),
                 vec![slant_left.clone()]
                ),
                /*
                     _ 
                      `- 
                   
                */
                (self.is_char(this, is_backtick)
                 && self.is_char(top_left, is_low_horizontal)
                 && self.is_char(right, is_horizontal),
                 vec![axay_excy.clone()]
                ),
                /*
                       __
                    --'
                   
                */
                (self.is_char(this, is_high_round)
                 && self.is_char(top_right, is_low_horizontal)
                 && self.is_char(left, is_horizontal),
                 vec![axcy_exay.clone()]
                ),
                /*
                     -._ 
                   
                */
                (self.is_char(this, is_low_round)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(right, is_low_horizontal),
                 vec![axcy_exey.clone()]
                ),
                /*
                     _.-
                   
                */
                (self.is_char(this, is_low_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(left, is_low_horizontal),
                 vec![axey_excy.clone()]
                ),
                /*
                    ^
                    |
                */
                (self.is_char(this, is_arrow_up)
                 && self.is_char(bottom, is_vertical),
                 vec![arrow_up.clone()]
                ),
                /*
                    ^
                    :
                */
                (self.is_char(this, is_arrow_up)
                 && self.is_char(bottom, is_vertical_dashed),
                 vec![arrow_up_dashed.clone()]
                ),
                /*
                    |
                    V
                */
                (self.is_char(this, is_arrow_down)
                 && self.is_char(top, is_vertical),
                 vec![arrow_down.clone()]
                ),

                /*
                    :
                    V
                */
                (self.is_char(this, is_arrow_down)
                 && self.is_char(top, is_vertical_dashed),
                 vec![arrow_down_dashed.clone()]
                ),
                /*
                    <-
                     
                */
                (self.is_char(this, is_arrow_left)
                 && self.is_char(right, is_horizontal),
                 vec![arrow_left.clone()]
                ),
                /*
                    <=
                     
                */
                (self.is_char(this, is_arrow_left)
                 && self.is_char(right, is_horizontal_dashed),
                 vec![arrow_left_dashed.clone()]
                ),
                /*
                    ->
                     
                */
                (self.is_char(this, is_arrow_right)
                 && self.is_char(left, is_horizontal),
                 vec![arrow_right.clone()]
                ),
                /*
                    =>
                     
                */
                (self.is_char(this, is_arrow_right)
                 && self.is_char(left, is_horizontal_dashed),
                 vec![arrow_right_dashed.clone()]
                ),
                /*
                    ^
                     \
                */
                (self.is_char(this, is_arrow_up)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![arrow_top_left.clone()]
                ),
                /*
                      ^
                     /
                */
                (self.is_char(this, is_arrow_up)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![arrow_top_right.clone()]
                ),
                /*
                      /
                     V 
                */
                (self.is_char(this, is_arrow_down)
                 && self.is_char(top_right, is_slant_right),
                 vec![arrow_bottom_left.clone()]
                ),
                /*
                      \
                       V 
                */
                (self.is_char(this, is_arrow_down)
                 && self.is_char(top_left, is_slant_left),
                 vec![arrow_bottom_right.clone()]
                ),
                /*
                       _  or |_
                      |
                */
                (self.is_char(this, is_low_horizontal)
                 && (self.is_char(bottom_left, is_vertical)
                    || self.is_char(left, is_vertical)
                    ),
                 vec![low_horizontal_extend_left_half.clone()]
                ),
                /*
                       _  or _|
                        |
                */
                (self.is_char(this, is_low_horizontal)
                 && (self.is_char(bottom_right, is_vertical)
                    || self.is_char(right, is_vertical)
                    ),
                 vec![low_horizontal_extend_right_half.clone()]
                ),
                /*
                      -| 
                       
                */
                (self.is_char(this, is_horizontal)
                 && self.is_char(right, is_vertical),
                 vec![axcy_exchcy.clone()]
                ),
                /*
                      |-
                       
                */
                (self.is_char(this, is_horizontal)
                 && self.is_char(left, is_vertical),
                 vec![excy_axchcy.clone()]
                ),
                /*
                       /_
                     
                */
                (self.is_char(this, is_low_horizontal)
                 && self.is_char(left, is_slant_right),
                 vec![low_horizontal_extend_left_full.clone()]
                ),
                /*
                       /_
                     
                */
                (self.is_char(this, is_slant_right)
                 && self.is_char(right, is_low_horizontal),
                 vec![slant_right.clone(), low_horizontal_extend_right_full.clone()] 
                ),
                /*
                       _\
                     
                */
                (self.is_char(this, is_low_horizontal)
                 && self.is_char(right, is_slant_left),
                 vec![low_horizontal_extend_right_full.clone()]
                ),
                /*
                       |
                       \
                     
                */
                (self.is_char(this, is_slant_left)
                 && self.is_char(top, is_vertical),
                 vec![cxcy_exey.clone()]
                ),
                /*
                       |
                       \
                     
                */
                (self.is_char(this, is_vertical)
                 && self.is_char(bottom, is_slant_left),
                 vec![cxay_cxeycv.clone()]
                ),
                /*
                       |
                       /
                     
                */
                (self.is_char(this, is_slant_right)
                 && self.is_char(top, is_vertical),
                 vec![cxcy_axey.clone()]
                ),
                /*
                       |
                       /
                     
                */
                (self.is_char(this, is_vertical)
                 && self.is_char(bottom, is_slant_right),
                 vec![cxay_cxeycv.clone()]
                ),
                /*
                      |
                       \
                     
                */
                (self.is_char(this, is_slant_left)
                 && self.is_char(top_left, is_vertical),
                 vec![exey_axchaycv.clone()]
                ),
                /*
                      |
                       \
                     
                */
                (self.is_char(this, is_vertical)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![cxay_cxcy.clone()]
                ),
                /*
                        |
                       /
                     
                */
                (self.is_char(this, is_slant_right)
                 && self.is_char(top_right, is_vertical),
                 vec![axey_exchaycv.clone()]
                ),
                /*
                        |
                       /
                     
                */
                (self.is_char(this, is_vertical)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![cxay_cxcy.clone()]
                ),
                /*
                       /
                       |
                     
                */
                (self.is_char(this, is_slant_right)
                 && self.is_char(bottom, is_vertical),
                 vec![cxcy_exay.clone()]
                ),
                /*
                       /
                       |
                     
                */
                (self.is_char(this, is_vertical)
                 && self.is_char(top, is_slant_right),
                 vec![cxaycv_cxey.clone()]
                ),
                /*
                       \
                       |
                     
                */
                (self.is_char(this, is_slant_left)
                 && self.is_char(bottom, is_vertical),
                 vec![axay_cxcy.clone()]
                ),
                /*
                       \
                       |
                     
                */
                (self.is_char(this, is_vertical)
                 && self.is_char(top, is_slant_left),
                 vec![cxaycv_cxey.clone()]
                ),
                /*
                       \
                        |
                     
                */
                (self.is_char(this, is_slant_left)
                 && self.is_char(bottom_right, is_vertical),
                 vec![axay_excheycv.clone()]
                ),
                /*
                       \
                        |
                     
                */
                (self.is_char(this, is_vertical)
                 && self.is_char(top_left, is_slant_left),
                 vec![cxcy_cxey.clone()]
                ),
                /*
                         /
                        |
                     
                */
                (self.is_char(this, is_slant_right)
                 && self.is_char(bottom_left, is_vertical),
                 vec![exay_axcheycv.clone()]
                ),
                /*
                         /
                        |
                     
                */
                (self.is_char(this, is_vertical)
                 && self.is_char(top_right, is_slant_right),
                 vec![cxcy_cxey.clone()]
                ),
                /*
                        ,     .
                      ,'    .'
                */
                (self.is_char(this, is_high_round)
                 &&((self.is_char(left, is_comma)
                      || self.is_char(left, is_low_round)
                    )
                ||(self.is_char(top_right, is_comma)
                     || self.is_char(top_right, is_low_round)
                  )
                ),
                 vec![exay_axcy.clone()]
                ),
                /*
                      ,'    .'
                     '     '
                */
                ((self.is_char(this, is_comma)
                    ||self.is_char(this, is_low_round)
                 )
                &&(self.is_char(right, is_high_round)
                   ||self.is_char(bottom_left, is_high_round)
                ),
                 vec![excy_axey.clone()]
                ),
                /*
                    `.
                      `
                */
                (self.is_char(this, is_period)
                 &&(self.is_char(bottom_right, is_backtick)
                 || self.is_char(left, is_backtick)),
                 vec![axcy_exey.clone()]
                ),
                /*
                    .
                     `.
                       
                */
                (self.is_char(this, is_backtick)
                 && (self.is_char(right, is_period)
                 || self.is_char(top_left, is_period)),
                 vec![axay_excy.clone()]
                ),
                /*
                      ,     .
                    /'    /'
                    
                */
                (self.is_char(this, is_high_round)
                 && (self.is_char(top_right, is_comma)
                    || self.is_char(top_right, is_low_round)
                    )
                 && self.is_char(left, is_slant_right),
                 vec![exay_axehey.clone()]
                ),
                /*
                    _
                     `.
                */
                (self.is_char(this, is_backtick)
                 && self.is_char(top_left, is_low_horizontal)
                 &&self.is_char(right, is_low_round),
                 vec![axay_excy.clone()]
                ),
                /*
                    speech bubble
                     .
                      `\
                */
                (self.is_char(this, is_backtick)
                 && self.is_char(top_left, is_low_round)
                 && self.is_char(right, is_slant_left),
                 vec![axay_exehey.clone()]
                ),
                /*
                    speech bubble
                     _
                      `\
                */
                (self.is_char(this, is_backtick)
                 && self.is_char(top_left, is_low_horizontal)
                 && self.is_char(right, is_slant_left),
                 vec![axay_exehey.clone()]
                ),
                /*
                      `.
                        \
                     
                */
                (self.is_char(this, is_low_round)
                 && self.is_char(left, is_backtick)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![arc_exdhey_axehay.clone()]
                ),
                /*
                      `.
                        \
                     
                */
                (self.is_char(this, is_backtick)
                 && self.is_char(right, is_low_round)
                 && self.is_char(&right.bottom_right(), is_slant_left),
                 vec![]
                ),
                /*
                      `.
                        \
                     
                */
                (self.is_char(this, is_slant_left)
                 && self.is_char(top_left, is_low_round)
                 && self.is_char(&top_left.left(), is_backtick),
                 vec![arc_exchey_dxay.clone()]
                ),
                /*
                    ,' 
                   /  
                     
                */
                (self.is_char(this, is_comma)
                 && self.is_char(right, is_high_round)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![arc_exehay_axdhey.clone()]
                ),
                /*
                    ,' 
                   /  
                     
                */
                (self.is_char(this, is_high_round)
                 && self.is_char(left, is_comma)
                 && self.is_char(&left.bottom_left(), is_slant_right),
                 vec![]
                ),
                /*
                    ,' 
                   /  
                     
                */
                (self.is_char(this, is_slant_right)
                 && self.is_char(top_right, is_comma)
                 && self.is_char(&top_right.right(), is_high_round),
                 vec![arc_bxay_axchey.clone()]
                ),
                /*
                   \    \
                    `.   ',
                     
                */
                ((self.is_char(this, is_high_round) 
                 || self.is_char(this, is_backtick)
                 )
                 && (self.is_char(right, is_low_round)
                    || self.is_char(right, is_comma)
                    )
                 && self.is_char(top_left, is_slant_left),
                 vec![arc_axdhay_exehey.clone()]
                ),
                /*
                   \    \
                    `.   ',
                     
                */
                ((self.is_char(left, is_high_round) 
                 || self.is_char(left, is_backtick)
                 )
                 && (self.is_char(this, is_low_round)
                    || self.is_char(this, is_comma)
                    )
                 && self.is_char(&left.top_left(), is_slant_left),
                 vec![]
                ),
                /*
                   \    \
                    `.   ',
                     
                */
                ((self.is_char(bottom_right, is_high_round) 
                 || self.is_char(bottom_right, is_backtick)
                 )
                 && (self.is_char(&bottom_right.right(), is_low_round)
                    || self.is_char(&bottom_right.right(), is_comma)
                    )
                 && self.is_char(this, is_slant_left),
                 vec![arc_axchay_bxey.clone()]
                ),
                /*
                       /   /
                     .'  ,'
                */
                (self.is_char(this, is_high_round)
                 && (self.is_char(left, is_low_round)
                    || self.is_char(left, is_comma)
                    )
                 && self.is_char(top_right, is_slant_right),
                 vec![arc_axehey_exdhay.clone()]
                ),
                /*
                       /   /
                     .'  ,'
                */
                (self.is_char(this, is_slant_right)
                 && (self.is_char(&bottom_left.left(), is_low_round)
                    || self.is_char(&bottom_left.left(), is_comma)
                    )
                 && self.is_char(bottom_left, is_high_round),
                 vec![arc_dxey_exchay.clone()]
                ),
                /*
                       /   /
                     .'  ,'
                */
                (self.is_char(&right.top_right(), is_slant_right)
                 && (self.is_char(this, is_low_round)
                    || self.is_char(this, is_comma)
                    )
                 && self.is_char(right, is_high_round),
                 vec![]
                ),
                /*
                    . ,   . ,   
                     '     `
                       
                */
                ((self.is_char(this, is_high_round)
                    ||self.is_char(this, is_backtick)
                 )
                 && self.is_char(top_right, is_comma)
                 && self.is_char(top_left, is_low_round),
                 vec![axay_cxby.clone(), cxby_exay.clone()]
                ),
                /*
                 
                  '.'  `.'
                 
                */
                (self.is_char(this, is_low_round)
                 && (self.is_char(left, is_high_round)
                    || self.is_char(left, is_backtick)
                    )
                 && self.is_char(right, is_high_round),
                 vec![axcy_cxdy.clone(), cxdy_excy.clone()]
                ),
                /*

                    .'    or  .'
                     `         '

                 */
                 (self.is_char(this, is_low_round)
                  && self.is_char(right, is_high_round)
                  && (self.is_char(bottom_right, is_high_round)
                     ||self.is_char(bottom_right, is_backtick)),
                  vec![cxdy_excy.clone(), cxdy_exey.clone()]
                 ),
                /*

                     `,   or   `.  or  '.
                     '         '       '

                 */
                 ((self.is_char(this, is_low_round)
                    || self.is_char(this, is_comma))
                  && (self.is_char(left, is_high_round)
                    || self.is_char(left, is_backtick))
                  && self.is_char(bottom_left, is_high_round),
                  vec![cxdy_axcy.clone(), cxdy_axey.clone()]
                 ),
                 /*
                      .       .  
                     ' `  or ' '  
                  
                  */
                 (self.is_char(this, is_low_round)
                  && self.is_char(bottom_left, is_high_round)
                  && (self.is_char(bottom_right, is_high_round)
                     || self.is_char(bottom_right, is_backtick)),
                  vec![cxdy_axey.clone(), cxdy_exey.clone()]
                 ),

                 /*
                     .'.  or    ,'.
                  
                  */
                 ((self.is_char(this, is_high_round)
                    || self.is_char(this, is_backtick))
                  && self.is_char(right, is_low_round)
                  && (self.is_char(left, is_low_round)
                     || self.is_char(left, is_comma)),
                  vec![cxby_axcy.clone(), cxby_excy.clone()]
                 ),

                /*
                      +-
                      | 
                */
                (self.is_char(this, is_intersection)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(bottom, is_vertical),
                 vec![cxcy_cxey.clone(), cxcy_excy.clone()]
                ),
                /*
                     -+
                      | 
                */
                (self.is_char(this, is_intersection)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(bottom, is_vertical),
                 vec![cxcy_cxey.clone(), cxcy_axcy.clone()]
                ),
                /*
                     |
                     +-
                */
                (self.is_char(this, is_intersection)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top, is_vertical),
                 vec![cxcy_cxay.clone(), cxcy_excy.clone()]
                ),
                /*
                     |
                    -+
                */
                (self.is_char(this, is_intersection)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top, is_vertical),
                 vec![cxcy_cxay.clone(), cxcy_axcy.clone()]
                ),
                /*
                      .-   ,-
                      |    |
                */
                ((self.is_char(this, is_round)
                 || self.is_char(this, is_comma)
                 )
                 && self.is_char(right, is_horizontal)
                 && self.is_char(bottom, is_vertical),
                 vec![cxdy_cxey.clone(), arc_excy_cxdy.clone()]
                ),
                /*
                      -.
                       | 
                */
                (self.is_char(this, is_round)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(bottom, is_vertical),
                 vec![cxdy_cxey.clone(), arc_cxdy_axcy.clone()]
                ),
                /*
                     |       |
                     '-      `-
                */
                ((self.is_char(this, is_round)
                  || self.is_char(this, is_backtick)
                  )
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top, is_vertical),
                 vec![cxay_cxby.clone(), arc_cxby_excy.clone()]
                ),
                /*
                     | 
                    -' 
                */
                (self.is_char(this, is_round)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top, is_vertical),
                 vec![cxay_cxby.clone(), arc_axcy_cxby.clone()]
                ),

                /*
                     | 
                    _' 
                */
                (self.is_char(this, is_round)
                 && self.is_char(left, is_low_horizontal)
                 && self.is_char(top, is_vertical),
                 vec![arc_axey_cxdy.clone(),cxdy_cxay.clone() ]
                ),
                /*
                     | 
                     '_
                */
                (self.is_char(this, is_round)
                 && self.is_char(right, is_low_horizontal)
                 && self.is_char(top, is_vertical),
                 vec![cxdy_cxay.clone(),arc_cxdy_exey.clone()]
                ),
                /*
                    .-  
                   / 
                */
                (self.is_char(this, is_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![axey_bxdy.clone(), arc_excy_bxdy.clone()]
                ),
                /*
                   -.  
                     \ 
                */
                (self.is_char(this, is_round)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![exey_dxdy.clone(), arc_dxdy_axcy.clone()]
                ),
                /*
                   -.  
                   / 
                */
                (self.is_char(this, is_round)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![axey_bxdy.clone(), arc_bxdy_axcy.clone()]
                ),
                /*
                   .-
                    \
                */
                (self.is_char(this, is_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![exey_dxdy.clone(), arc_excy_dxdy.clone()]
                ),
                /*
                   \  
                    '-  
                */
                (self.is_char(this, is_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top_left, is_slant_left),
                 vec![axay_bxby.clone(), arc_bxby_excy.clone()]
                ),
                /*
                     / 
                    '-  
                */
                (self.is_char(this, is_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top_right, is_slant_right),
                 vec![dxby_exay.clone(), arc_dxby_excy.clone()]
                ),
                /*
                    \
                    -'
                */
                (self.is_char(this, is_round)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top_left, is_slant_left),
                 vec![axay_bxby.clone(), arc_axcy_bxby.clone()]
                ),
                /*
                      /
                    -'
                */
                (self.is_char(this, is_round)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top_right, is_slant_right),
                 vec![dxby_exay.clone(), arc_axcy_dxby.clone()]
                ),
                /*
                    \       \
                     .  or   )
                    /       /
                */
                ((self.is_char(this, is_round) || self.is_char(this, is_close_curve))
                 && self.is_char(top_left, is_slant_left)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![axay_bxby.clone(),axey_bxdy.clone(), arc_bxdy_bxby.clone()]
                ),
                /*
                      /       /
                     .  or   (
                      \       \
                */
                ((self.is_char(this, is_round) || self.is_char(this, is_open_curve))
                 && self.is_char(top_right, is_slant_right)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![exay_dxby.clone(),exey_dxdy.clone(), arc_dxby_dxdy.clone()]
                ),
                /*
                      .      .     ,
                     (  or  (  or (
                      '      `     `
                */  
                (self.is_char(this, is_open_curve) 
                 && (self.is_char(top_right, is_round)
                    || self.is_char(top_right, is_comma)
                    )
                 && (self.is_char(bottom_right, is_high_round)
                   || self.is_char(bottom_right, is_backtick)
                   ),
                 vec![arc_dxay_dxey.clone()]
                ),
                /*
                      .
                       ) 
                      '
                */
                (self.is_char(this, is_close_curve) 
                 && self.is_char(top_left, is_round)
                 && self.is_char(bottom_left, is_round),
                 vec![arc_bxey_bxay.clone()]
                ),
                /*
                      .-          ,-
                     (     or    (
                */
                ((self.is_char(this, is_low_round) 
                  || self.is_char(this, is_comma)
                  )
                 && self.is_char(right, is_horizontal)
                 && self.is_char(bottom_left, is_open_curve),
                 vec![arc_excy_axbhey.clone()]
                ),
                /*
                       -.
                         ) 
                */
                (self.is_char(this, is_round) 
                 && self.is_char(left, is_horizontal)
                 && self.is_char(bottom_right, is_close_curve),
                 vec![arc_exbhey_axcy.clone()]
                ),
                /*
                    (    or   (
                     '-        `-
                */
                ((self.is_char(this, is_high_round) 
                  || self.is_char(this, is_backtick)
                 )
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top_left, is_open_curve),
                 vec![arc_axbhay_excy.clone()]
                ),
                /*
                        ) 
                      -'
                */
                (self.is_char(this, is_round) 
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top_right, is_close_curve),
                 vec![arc_axcy_exbhay.clone()]
                ),
                /*
                     .- 
                     ' 
                */
                (self.is_char(this, is_low_round) 
                 && self.is_char(right, is_horizontal)
                 && self.is_char(bottom, is_high_round),
                 vec![arc_excy_cxdy.clone(), cxdy_cxey.clone()]
                ),
                /*
                     -.
                      ' 
                */
                (self.is_char(this, is_low_round) 
                 && self.is_char(left, is_horizontal)
                 && self.is_char(bottom, is_high_round),
                 vec![arc_cxdy_axcy.clone(),cxdy_cxey.clone()]
                ),
                /*
                     . 
                     '-
                */
                (self.is_char(this, is_high_round) 
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top, is_low_round),
                 vec![arc_cxby_excy.clone(), cxay_cxby.clone()]
                ),
                /*
                       . 
                      -'
                */
                (self.is_char(this, is_high_round) 
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top, is_low_round),
                 vec![arc_axcy_cxby.clone(), cxay_cxby.clone()]
                ),
                /*
                      .-.  
                         
                */
                (self.is_char(this, is_horizontal) 
                 && self.is_char(left, is_low_round)
                 && self.is_char(right, is_low_round),
                 vec![arc_excy_axcy.clone()]
                ),
                /*
                      ._.  
                         
                */
                (self.is_char(this, is_low_horizontal) 
                 && self.is_char(left, is_low_round)
                 && self.is_char(right, is_low_round),
                 vec![arc_axey_exey.clone()]
                ),
                /*
                      .
                      .'
                         
                */
                (self.is_char(this, is_high_round) 
                 && self.is_char(left, is_low_round)
                 && self.is_char(top_left, is_low_round),
                 vec![arc_axcy_axay.clone()]
                ),
                /*
                      .-.  
                     (    
                */
                (self.is_char(this, is_horizontal) 
                 && self.is_char(left, is_low_round)
                 && self.is_char(right, is_low_round)
                 && self.is_char(&this.bottom_left().left(), is_open_curve),
                 vec![arc_excy_axcy.clone()]
                ),
                /*         
                     (     or  (
                      '-'       `-'
                */
                (self.is_char(this, is_horizontal) 
                 && (self.is_char(left, is_high_round)
                    || self.is_char(left, is_backtick)
                    )
                 && self.is_char(right, is_high_round)
                 && self.is_char(&this.top_left().left(), is_open_curve),
                 vec![arc_axcy_excy.clone()]
                ),
                /*
                      / 
                     .  
                     |
                */
                (self.is_char(this, is_round)
                 && self.is_char(bottom, is_vertical)
                 && self.is_char(top_right, is_slant_right),
                 vec![exay_dxby.clone(),cxey_cxdy.clone(), arc_dxby_cxdy.clone()]
                ),
                /*
                     | 
                     .  
                    /
                */
                (self.is_char(this, is_round)
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![cxay_cxby.clone(), axey_bxdy.clone(), arc_bxdy_cxby.clone()]
                ),
                /*
                    \
                     .  
                     | 
                */
                (self.is_char(this, is_round)
                 && self.is_char(bottom, is_vertical)
                 && self.is_char(top_left, is_slant_left),
                 vec![axay_bxby.clone(), cxdy_cxey.clone(), arc_cxdy_bxby.clone()]
                ),
                /*
                     |
                     .  
                      \ 
                */
                (self.is_char(this, is_round)
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![cxay_cxby.clone(), dxdy_exey.clone(), arc_cxby_dxdy.clone()]
                ),
                /*
                     .       
                    / \       
                */
                (self.is_char(this, is_low_round)
                 && self.is_char(bottom_left, is_slant_right)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![axey_cxcy.clone(), cxcy_exey.clone()]
                ),
                /*
                       \ /
                        '

                */
                (self.is_char(this, is_high_round)
                 && self.is_char(top_left, is_slant_left)
                 && self.is_char(top_right, is_slant_right),
                 vec![axay_cxcy.clone(), cxcy_exay.clone()]
                ),
                /*
                     |  
                    / \
                */
                (self.is_char(this, is_vertical)
                 && self.is_char(bottom_left, is_slant_right)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![axey_cxcy.clone(), cxcy_exey.clone(), cxay_cxcy.clone()]
                ),

                /*
                    \ / 
                     | 
                */
                (self.is_char(this, is_vertical)
                 && self.is_char(top_left, is_slant_left)
                 && self.is_char(top_right, is_slant_right),
                 vec![axay_cxcy.clone(), cxcy_exay.clone(), cxey_cxcy.clone()]
                ),
                /*
                     .  
                     |\
                */
                (self.is_char(this, is_round)
                 && self.is_char(bottom, is_vertical)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![cxcy_cxey.clone(), cxcy_exey.clone()]
                ),
                /*
                      .  
                     /|
                */
                (self.is_char(this, is_round)
                 && self.is_char(bottom, is_vertical)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![cxcy_cxey.clone(), cxcy_axey.clone()]
                ),
                /*
                     \|  
                      '
                */
                (self.is_char(this, is_round)
                 && self.is_char(top, is_vertical)
                 && self.is_char(top_left, is_slant_left),
                 vec![axay_cxcy.clone(), cxcy_cxay.clone()]
                ),
                /*
                      |/
                      '
                */
                (self.is_char(this, is_round)
                 && self.is_char(top, is_vertical)
                 && self.is_char(top_right, is_slant_right),
                 vec![cxay_cxcy.clone(), cxcy_exay.clone()]
                ),
                /*
                     -.
                      (
                      
                */
                (self.is_char(this, is_low_round)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(bottom, is_open_curve),
                 vec![arc_cxey_axcy.clone()]
                ),
                /*
                     
                      (
                     -'
                      
                */
                (self.is_char(this, is_high_round)
                 && self.is_char(top, is_open_curve)
                 && self.is_char(left, is_horizontal),
                 vec![arc_axcy_cxay.clone()]
                ),
                /*
                     .-
                     )
                      
                */
                (self.is_char(this, is_low_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(bottom, is_close_curve),
                 vec![arc_excy_cxey.clone()]
                ),
                /*
                    )
                    '-
                      
                */
                (self.is_char(this, is_high_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top, is_close_curve),
                 vec![arc_cxay_excy.clone()]
                ),
                /*
                      
                      (
                       >
                      
                */
                (self.is_char(this, is_open_curve)
                 && self.is_char(bottom_right, is_arrow_right),
                 vec![arc_cxay_exey.clone()]
                ),
                /*
                      
                       >
                      (
                      
                */
                (self.is_char(this, is_open_curve)
                 && self.is_char(top_right, is_arrow_right),
                 vec![arc_exay_cxey.clone()]
                ),
                /*
                    expandable close bracket
                      
                      (
                       >
                      (
                      
                */
                (self.is_char(this, is_arrow_right)
                 && self.is_char(top_left, is_open_curve)
                 && self.is_char(bottom_left, is_open_curve),
                 vec![axay_excy.clone(), axey_excy.clone()]
                ),
                /*
                      
                      )
                     <
                      
                */
                (self.is_char(this, is_close_curve)
                 && self.is_char(bottom_left, is_arrow_left),
                 vec![arc_axey_cxay.clone()]
                ),
                /*
                      
                     < 
                      )
                      
                */
                (self.is_char(this, is_close_curve)
                 && self.is_char(top_left, is_arrow_left),
                 vec![arc_cxey_axay.clone()]
                ),
                /*
                    expandable open brcket
                      
                      )
                     <
                      )
                      
                */
                (self.is_char(this, is_arrow_left)
                 && self.is_char(top_right, is_close_curve)
                 && self.is_char(bottom_right, is_close_curve),
                 vec![axcy_exay.clone(), axcy_exey.clone()]
                ),

                /*
                     .- 
                    < 
                */
                (self.is_char(this, is_low_round) 
                 && self.is_char(right, is_horizontal)
                 && self.is_char(bottom_left, is_arrow_left),
                 vec![arc_excy_cxdy.clone(), cxdy_cxeybv.clone()]
                ),
                /*
                    left speech balloon pointer  
                      .
                     <
                      '
                      
                */
                (self.is_char(this, is_arrow_left)
                 && self.is_char(top_right, is_low_round)
                 && self.is_char(bottom_right, is_high_round),
                 vec![axcy_exchby.clone(), axcy_exchdy.clone()]
                ),
                /*
                    <  
                     '-
                */
                (self.is_char(this, is_high_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top_left, is_arrow_left),
                 vec![arc_cxby_excy.clone(), cxaybv_cxby.clone()]
                ),
                /*
                    right speech balloon pointer  
                      .
                       >
                      '
                      
                */
                (self.is_char(this, is_arrow_right)
                 && self.is_char(top_left, is_low_round)
                 && self.is_char(bottom_left, is_high_round),
                 vec![axchby_excy.clone(), axchdy_excy.clone()]
                ),

                /*
                      > 
                    -'
                */
                (self.is_char(this, is_high_round)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top_right, is_arrow_right),
                 vec![arc_axcy_cxby.clone(), cxaybv_cxby.clone()]
                ),
                /*

                    -. 
                      > 
                */
                (self.is_char(this, is_low_round) 
                 && self.is_char(left, is_horizontal)
                 && self.is_char(bottom_right, is_arrow_right),
                 vec![arc_cxdy_axcy.clone(), cxdy_cxeybv.clone()]
                ),
                /*
                      |_\
                      
                */
                (self.is_char(this, is_low_horizontal)
                 && self.is_char(left, is_vertical)
                 && self.is_char(right, is_slant_left),
                 vec![axchey_exehey.clone()]
                ),
                /*
                     /
                    .-
                   /
                */
                (self.is_char(this, is_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top_right, is_slant_right)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![slant_right.clone(), dxcy_excy.clone(), arc_dxcy_bxdy.clone()]
                ),
                /*
                     /
                   -.
                   /
                */
                (self.is_char(this, is_round)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top_right, is_slant_right)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![slant_right.clone(), bxcy_axcy.clone(), arc_bxcy_dxby.clone()]
                ),
                /*
                    \
                    -.
                      \
                   
                */
                (self.is_char(this, is_round)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top_left, is_slant_left)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![slant_left.clone(),bxcy_axcy.clone(), arc_dxdy_bxcy.clone()]
                ),
                /*
                    \
                     .-
                      \
                   
                */
                (self.is_char(this, is_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top_left, is_slant_left)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![slant_left.clone(),dxcy_excy.clone(), arc_bxby_dxcy.clone()]
                ),
                /*
                    
                    -.-
                    / 
                   
                */
                (self.is_char(this, is_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![horizontal.clone(), axey_bxdy.clone(), arc_excy_bxdy.clone()]
                ),
                /*
                    
                    -.-
                      \ 
                   
                */
                (self.is_char(this, is_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![horizontal.clone(), dxdy_exey.clone(), arc_dxdy_axcy.clone()]
                ),
                /*
                      /
                    -'-
                   
                */
                (self.is_char(this, is_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top_right, is_slant_right),
                 vec![horizontal.clone(), dxby_exay.clone(), arc_axcy_dxby.clone()]
                ),
                /*
                    \ 
                    -'-
                   
                */
                (self.is_char(this, is_round)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top_left, is_slant_left),
                 vec![horizontal.clone(), axay_bxby.clone(), arc_bxby_excy.clone()]
                ),
                /*
                     |
                    -+-
                */
                (self.is_char(this, is_intersection)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top, is_vertical),
                 vec![cxcy_cxay.clone(), horizontal.clone()]
                ),
                /*
                    -+-
                     |
                */
                (self.is_char(this, is_intersection)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(bottom, is_vertical),
                 vec![cxcy_cxey.clone(), horizontal.clone()]
                ),
                /*
                     |
                    -+
                     |
                */
                (self.is_char(this, is_intersection)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom, is_vertical),
                 vec![vertical.clone(), cxcy_axcy.clone()]
                ),
                /*
                     |
                     +-
                     |
                */
                (self.is_char(this, is_intersection)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom, is_vertical),
                 vec![vertical.clone(), cxcy_excy.clone()]
                ),
                /*
                     | 
                     .  
                    /|
                */
                (self.is_char(this, is_round)
                 && self.is_char(bottom, is_vertical)
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom_left, is_slant_right),
                 vec![vertical.clone(), axey_bxdy.clone(), arc_bxdy_cxby.clone()]
                ),
                /*
                     | 
                     .  
                     |\
                */
                (self.is_char(this, is_round)
                 && self.is_char(bottom, is_vertical)
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![vertical.clone(), exey_dxdy.clone(), arc_cxby_dxdy.clone()]
                ),
                /*
                     | 
                     .  
                    / \
                */
                (self.is_char(this, is_round)
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom_left, is_slant_right)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![cxay_cxcy.clone(), cxcy_exey.clone(), cxcy_axey.clone()]
                ),
                /*
                     |/
                     '  
                     | 
                */
                (self.is_char(this, is_round)
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom, is_vertical)
                 && self.is_char(top_right, is_slant_right),
                 vec![vertical.clone(), cxcy_exay.clone()]
                ),
                /*
                    \|
                     '  
                     | 
                */
                (self.is_char(this, is_round)
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom, is_vertical)
                 && self.is_char(top_left, is_slant_left),
                 vec![vertical.clone(), axay_cxcy.clone()]
                ),
                /*
                    |  
                   -+-
                    | 
                */
                ((self.is_char(this, is_intersection) || self.is_char(this, is_round) || self.is_char(this, is_marker))
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom, is_vertical)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(right, is_horizontal),
                 vec![vertical.clone(), horizontal.clone()]
                ),
                /*
                    :  
                   =+=
                    : 
                */
                ((self.is_char(this, is_intersection) || self.is_char(this, is_round) || self.is_char(this, is_marker))
                 && self.is_char(top, is_vertical_dashed)
                 && self.is_char(bottom, is_vertical_dashed)
                 && self.is_char(left, is_horizontal_dashed)
                 && self.is_char(right, is_horizontal_dashed),
                 vec![vertical_dashed.clone(), horizontal_dashed.clone()]
                ),
                /*
                   \|/ 
                    + 
                   /|\
                */
                ((self.is_char(this, is_intersection) || self.is_char(this, is_round) || self.is_char(this, is_marker))
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom, is_vertical)
                 && self.is_char(top_left, is_slant_left)
                 && self.is_char(top_right, is_slant_right)
                 && self.is_char(bottom_left, is_slant_right)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![vertical.clone(), slant_left.clone(), slant_right.clone()]
                ),
                /*
                   \|/ 
                   -+-
                   /|\
                */
                ((self.is_char(this, is_intersection) || self.is_char(this, is_round) || self.is_char(this, is_marker))
                 && self.is_char(top, is_vertical)
                 && self.is_char(bottom, is_vertical)
                 && self.is_char(left, is_horizontal)
                 && self.is_char(right, is_horizontal)
                 && self.is_char(top_left, is_slant_left)
                 && self.is_char(top_right, is_slant_right)
                 && self.is_char(bottom_left, is_slant_right)
                 && self.is_char(bottom_right, is_slant_left),
                 vec![vertical.clone(), horizontal.clone(), slant_left.clone(), slant_right.clone()]
                ),
            ];
        let match_path: Option<(bool, Vec<Element>)> = match_list.into_iter()
            .rev()
            .find(|x| {
                let &(cond, _) = x;
                cond
            });

        let paths: Option<Vec<Element>> = match match_path {
            Some((_, paths)) => Some(paths),
            None => {
                let ch = self.get(this);
                match ch {
                    Some(ch) => {
                        if !(ch.string == " ") ||
                           (ch.string == " " && self.is_char(left, is_alphanumeric) &&
                            self.is_char(right, is_alphanumeric)) {
                            let s = escape_char(&ch.string);
                            let text = Element::Text(this.clone(), s);
                            Some(vec![text])
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            }
        };

        paths

    }


    fn get_all_elements(&self, settings: &Settings) -> Vec<(Loc, Vec<Element>)> {
        fn get_line_width(line: &Vec<GChar>) -> usize{
            let mut total_width = 0;
            for gch in line{
               total_width += gch.width;
            }
            total_width
        }
        let mut all_paths = vec![];
        for row in 0..self.lines.len() {
            let line = &self.lines[row];
            let line_width = get_line_width(line);
            for column in 0..line_width {
                let x = column as isize;
                let y = row as isize;
                match self.get_elements(x, y, settings) {
                    Some(paths) => {
                        all_paths.push((Loc::new(x, y), paths));
                    }
                    None => {
                        ();
                    }
                }
            }
        }
        all_paths
    }

    // each component has its relative location retain
    // use this info for optimizing svg by checking closest neigbor
    fn get_svg_nodes(&self, settings: &Settings) -> Vec<SvgElement> {
        let mut nodes = vec![];
        let elements = self.get_all_elements(settings);
        let input = if settings.optimize {
            let optimizer = Optimizer::new(elements);
            let optimized_elements = optimizer.optimize(settings);
            optimized_elements
        } else {
            elements.into_iter().flat_map(|(_, elm)| elm).collect()
        };
        for elem in input {
            let element = elem.to_svg(settings);
            nodes.push(element);
        }
        nodes
    }


    /// get the generated svg according to the settings specified
    pub fn get_svg(&self, settings: &Settings) -> SVG {
        let nodes = self.get_svg_nodes(settings);
        let width = settings.text_width * (self.columns + 4) as f32;
        let height = settings.text_height * (self.rows + 2)as f32;
        let mut svg = SVG::new()
            .set("font-size", 14)
            .set("font-family",
                "arial"
                )
            .set("width", width)
            .set("height", height);

        svg.append(get_defs());
        svg.append(get_styles());

        for node in nodes {
            match node {
                SvgElement::Circle(circle) => {
                    svg.append(circle);
                }
                SvgElement::Line(line) => {
                    svg.append(line);
                }
                SvgElement::Path(path) => {
                    svg.append(path);
                }
                SvgElement::Text(text) => {
                    svg.append(text);
                }
            }
        }
        svg
    }
}

fn get_defs() -> Definitions {
    let mut defs = Definitions::new();
    defs.append(arrow_marker());
    defs
}

fn get_styles() -> Style {
    let style = r#"
    line, path {
      stroke: black;
      stroke-width: 2;
      stroke-opacity: 1;
      fill-opacity: 1;
      stroke-linecap: round;
      stroke-linejoin: miter;
    }
    circle {
      stroke: black;
      stroke-width: 2;
      stroke-opacity: 1;
      fill-opacity: 1;
      stroke-linecap: round;
      stroke-linejoin: miter;
      fill:white;
    }
    circle.solid {
      fill:black;
    }
    circle.open {
      fill:white;
    }
    tspan.head{
        fill: none;
        stroke: none;
    }
    "#;
    Style::new(style)
}

fn arrow_marker() -> Marker {
    let mut marker = Marker::new()
        .set("id", "triangle")
        .set("viewBox", "0 0 50 20")
        .set("refX", 15)
        .set("refY", 10)
        .set("markerUnits", "strokeWidth")
        .set("markerWidth", 10)
        .set("markerHeight", 10)
        .set("orient", "auto");

    let path = SvgPath::new().set("d", "M 0 0 L 30 10 L 0 20 z");
    marker.append(path);
    marker

}

fn is_vertical(ch: &str) -> bool {
    ch == "|"
}

fn is_horizontal(ch: &str) -> bool {
    ch == "-"
}

fn is_horizontal_dashed(ch: &str) -> bool {
    ch == "="
}

fn is_vertical_dashed(ch: &str) -> bool {
    ch == ":"
}

fn is_low_horizontal(ch: &str) -> bool {
    ch == "_"
}

fn is_low_horizontal_dashed(ch: &str) -> bool {
    ch == "."
}

fn is_slant_left(ch: &str) -> bool {
    ch == "\\"
}
fn is_slant_right(ch: &str) -> bool {
    ch == "/"
}

fn is_low_round(ch: &str) -> bool {
    ch == "."
}

fn is_period(ch: &str) -> bool{
    ch == "."
}

fn is_comma(ch: &str) -> bool {
    ch == ","
}

fn is_high_round(ch: &str) -> bool {
    ch == "\'"
}

fn is_backtick(ch: &str) -> bool {
    ch == "`"
}

fn is_round(ch: &str) -> bool {
    is_low_round(ch) || is_high_round(ch) || is_backtick(ch) || is_comma(ch)
}

fn is_intersection(ch: &str) -> bool {
    ch == "+"
}

fn is_marker(ch: &str) -> bool {
    ch == "*"
}

fn is_asterisk(ch: &str) -> bool {
    ch == "*"
}

fn is_o(ch: &str) -> bool {
    ch == "o"
}

fn is_arrow_up(ch: &str) -> bool {
    ch == "^"
}

fn is_arrow_down(ch: &str) -> bool {
    ch == "v" || ch == "V"
}

fn is_arrow_left(ch: &str) -> bool {
    ch == "<"
}

fn is_arrow_right(ch: &str) -> bool {
    ch == ">"
}

fn is_open_curve(ch: &str) -> bool {
    ch == "("
}

fn is_close_curve(ch: &str) -> bool {
    ch == ")"
}


fn escape_char(ch: &str) -> String {
    let escs = [("\"", "&quot;"), ("'", "&apos;"), ("<", "&lt;"), (">", "&gt;"), ("&", "&amp;")];
    let quote_match: Option<&(&str, &str)> = escs.iter()
        .find(|pair| {
            let &(e, _) = *pair;
            e == ch
        });
    let quoted: String = match quote_match {
        Some(&(_, quoted)) => String::from(quoted),
        None => {
            let mut s = String::new();
            s.push_str(&ch);
            s
        }
    };
    quoted

}


fn is_alphanumeric(ch:&str) -> bool{
    ch.chars().all(|c| c.is_alphanumeric())
}

#[test]
fn test_bob(){
    println!(r#"<meta charset="utf-8"/>"#);
    println!("<pre>");
    let bob = "|mmu--文件系统---- 调度器--------------4";
    println!("total width: {}", UnicodeWidthStr::width(bob));
    let mut acc_width = 0;
    for ch in bob.chars(){
        let uwidth =  UnicodeWidthChar::width(ch).unwrap();
        println!("[{}] {}",acc_width, ch);
        acc_width += uwidth;
    }
    let grid = Grid::from_str(bob);
    let loc = &Loc::new(39,0);
    let c = grid.get(loc);
    println!("{:?} {:?}", loc, c);
    println!("{:?}", grid);
    let svg = grid.get_svg(&Settings::no_optimization());
    println!("svg:{}", svg);
    assert_eq!(c, Some(&GChar::from_str("4")));
}

#[test]
fn test_meme(){
    let meme = r#"[( ͡° ͜ʖ ͡°)]ﾟ"#;
    println!(r#"<meta charset="utf-8"/>"#);
    println!("char count {}", meme.chars().count());
    println!("total bytes size {}", meme.len());
    println!("total width {}", UnicodeWidthStr::width(&*meme));
    for m in meme.chars(){
        println!("{} {} width:{} alphanumeric {}",m, m as u32, m.width().unwrap(), m.is_alphanumeric());
    }
    println!("<pre>");
    println!("{}", meme);
    let grid = Grid::from_str(meme);
    println!("{:#?}",grid);
    assert_eq!(grid.get(&Loc::new(6,0)), Some(&GChar::from_str(" ͡°")));
}

#[test]
fn test_eye_brow(){
    let meme = r#" ͡°"#;
    println!(r#"<meta charset="utf-8"/>"#);
    println!("char count {}", meme.chars().count());
    println!("total bytes size {}", meme.len());
    println!("total width {}", UnicodeWidthStr::width(&*meme));
    for m in meme.chars(){
        println!("{} {} width:{}",m, m as u32, m.width().unwrap());
    }
    println!("<pre>");
    println!("{}", meme);
    let grid = Grid::from_str(meme);
    println!("{:?}",grid);
    let ch = grid.get(&Loc::new(0,0));
    if let Some(ch) = ch{
        for ch in ch.string.chars(){
            println!("ch: {:?}", ch as u32);
        }
    }
    assert_eq!(ch, Some(&GChar::from_str(" ͡°")));
}


