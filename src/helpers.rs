use image::GenericImageView;
use image::{self, Pixel};
use image::{DynamicImage, ImageBuffer, Primitive, Rgba32FImage};
use num::{NumCast, ToPrimitive};

#[allow(dead_code)]
#[inline]
pub fn clamp<N>(a: N, min: N, max: N) -> N
where
    N: PartialOrd,
{
    if a < min {
        min
    } else if a > max {
        max
    } else {
        a
    }
}

pub struct Filter<'a> {
    // Expera o resultado da equacao do fitro a ser aplicado
    pub kernel: Box<dyn Fn(f32) -> f32 + 'a>,
    // O tamanho no qual este filtro e aplicado
    pub support: f32,
}

struct FloatNearest(f32);

impl ToPrimitive for FloatNearest {
    fn to_i8(&self) -> Option<i8> {
        self.0.round().to_i8()
    }
    fn to_i16(&self) -> Option<i16> {
        self.0.round().to_i16()
    }
    fn to_i64(&self) -> Option<i64> {
        self.0.round().to_i64()
    }
    fn to_u8(&self) -> Option<u8> {
        self.0.round().to_u8()
    }
    fn to_u16(&self) -> Option<u16> {
        self.0.round().to_u16()
    }
    fn to_u64(&self) -> Option<u64> {
        self.0.round().to_u64()
    }
    fn to_f64(&self) -> Option<f64> {
        self.0.to_f64()
    }
}
// Redimensiona a imagem na vertical com a funcao fornecida
fn vertical_sample<I, P, S>(image: &I, new_height: u32, filter: &mut Filter) -> Rgba32FImage
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (width, height) = image.dimensions();
    let mut out = ImageBuffer::new(width, new_height);

    let ratio = height as f32 / new_height as f32;
    let sratio = if ratio < 1.0 { 1.0 } else { ratio };
    let src_support: f32 = filter.support * sratio;

    for outy in 0..new_height {
        let inputy = (outy as f32 + 0.5) * ratio;

        let left = (inputy - src_support).floor() as i64;
        let left = clamp(left, 0, <i64 as From<_>>::from(height) - 1) as u32;

        let right = (inputy + src_support).ceil() as i64;
        let right = clamp(
            right,
            <i64 as From<_>>::from(left) + 1,
            <i64 as From<_>>::from(height),
        ) as u32;

        let inputy = inputy - 0.5;

        let mut ws = Vec::new();
        let mut sum = 0.0;
        for i in left..right {
            let w = (filter.kernel)((i as f32 - inputy) / sratio);
            ws.push(w);
            sum += w;
        }
        ws.iter_mut().for_each(|w| *w /= sum);

        for x in 0..width {
            let mut t = (0.0, 0.0, 0.0, 0.0);

            for (i, w) in ws.iter().enumerate() {
                let p = image.get_pixel(x, left + i as u32);

                #[allow(deprecated)]
                let (k1, k2, k3, k4) = p.channels4();
                let vec: (f32, f32, f32, f32) = (
                    NumCast::from(k1).unwrap(),
                    NumCast::from(k2).unwrap(),
                    NumCast::from(k3).unwrap(),
                    NumCast::from(k4).unwrap(),
                );

                t.0 += vec.0 * w;
                t.1 += vec.1 * w;
                t.2 += vec.2 * w;
                t.3 += vec.3 * w;
            }

            #[allow(deprecated)]
            // This is not necessarily Rgba.
            let t = Pixel::from_channels(t.0, t.1, t.2, t.3);

            out.put_pixel(x, outy, t);
        }
    }

    out
}
// Redimensiona a imagem na horizontal com a funcao fornecida
fn horizontal_sample<P, S>(
    image: &Rgba32FImage,
    new_width: u32,
    filter: &mut Filter,
) -> ImageBuffer<P, Vec<S>>
where
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (width, height) = image.dimensions();
    let mut out = ImageBuffer::new(new_width, height);

    let max: f32 = NumCast::from(S::DEFAULT_MAX_VALUE).unwrap();
    let min: f32 = NumCast::from(S::DEFAULT_MIN_VALUE).unwrap();
    let ratio = width as f32 / new_width as f32;
    let sratio = if ratio < 1.0 { 1.0 } else { ratio };
    let src_support = filter.support * sratio;

    for outx in 0..new_width {
        let inputx = (outx as f32 + 0.5) * ratio;

        let left = (inputx - src_support).floor() as i64;
        let left = clamp(left, 0, <i64 as From<_>>::from(width) - 1) as u32;

        let right = (inputx + src_support).ceil() as i64;
        let right = clamp(
            right,
            <i64 as From<_>>::from(left) + 1,
            <i64 as From<_>>::from(width),
        ) as u32;

        let inputx = inputx - 0.5;

        let mut ws = Vec::new();
        let mut sum = 0.0;
        for i in left..right {
            let w = (filter.kernel)((i as f32 - inputx) / sratio);
            ws.push(w);
            sum += w;
        }
        ws.iter_mut().for_each(|w| *w /= sum);

        for y in 0..height {
            let mut t = (0.0, 0.0, 0.0, 0.0);

            for (i, w) in ws.iter().enumerate() {
                let p = image.get_pixel(left + i as u32, y);

                #[allow(deprecated)]
                let vec = p.channels4();

                t.0 += vec.0 * w;
                t.1 += vec.1 * w;
                t.2 += vec.2 * w;
                t.3 += vec.3 * w;
            }

            let t = *Pixel::from_slice(&[
                NumCast::from(FloatNearest(clamp(t.0, min, max))).unwrap(),
                NumCast::from(FloatNearest(clamp(t.1, min, max))).unwrap(),
                NumCast::from(FloatNearest(clamp(t.2, min, max))).unwrap(),
                NumCast::from(FloatNearest(clamp(t.3, min, max))).unwrap(),
            ]);

            out.put_pixel(outx, y, t);
        }
    }

    out
}

pub fn resample<I, P, S>(
    image: &I,
    new_height: u32,
    new_width: u32,
    filter: &mut Filter,
) -> ImageBuffer<P, Vec<S>>
where
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
    I: GenericImageView<Pixel = P>,
{
    let i = vertical_sample(image, new_height, filter);
    horizontal_sample(&i, new_width, filter)
}

pub fn filter3x3<I, P, S>(image: &I, kernel: &[f32], scale: Option<f32>) -> ImageBuffer<P, Vec<S>>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let scale = scale.unwrap_or(1.0);
    // The kernel's input positions relative to the current pixel.
    let taps: &[(isize, isize)] = &[
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (0, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    let (width, height) = image.dimensions();

    let mut out = ImageBuffer::new(width, height);

    let max = S::DEFAULT_MAX_VALUE;
    let max: f32 = NumCast::from(max).unwrap();

    let sum = match kernel.iter().fold(0.0, |s, &item| s + item) {
        x if x == 0.0 => 1.0,
        sum => sum,
    };
    let sum = (sum, sum, sum, sum);

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let mut t = (0.0, 0.0, 0.0, 0.0);

            // TODO: There is no need to recalculate the kernel for each pixel.
            // Only a subtract and addition is needed for pixels after the first
            // in each row.
            for (&k, &(a, b)) in kernel.iter().zip(taps.iter()) {
                let k = (k, k, k, k);
                let x0 = x as isize + a;
                let y0 = y as isize + b;

                let p = image.get_pixel(x0 as u32, y0 as u32);

                #[allow(deprecated)]
                let (k1, k2, k3, k4) = p.channels4();

                let vec: (f32, f32, f32, f32) = (
                    NumCast::from(k1).unwrap(),
                    NumCast::from(k2).unwrap(),
                    NumCast::from(k3).unwrap(),
                    NumCast::from(k4).unwrap(),
                );

                t.0 += vec.0 * k.0;
                t.1 += vec.1 * k.1;
                t.2 += vec.2 * k.2;
                t.3 += vec.3 * k.3;
            }

            let (mut t1, mut t2, mut t3, mut t4) = (t.0 / sum.0, t.1 / sum.1, t.2 / sum.2, t.3 / sum.3);
            if scale != 0.0 {
                t1 = t1/scale;

                t2 = t2/scale;
                t3 = t3/scale;
                t4 = t4/scale;
            }
            #[allow(deprecated)]
            let t = Pixel::from_channels(
                NumCast::from(clamp(t1, 0.0, max)).unwrap(),
                NumCast::from(clamp(t2, 0.0, max)).unwrap(),
                NumCast::from(clamp(t3, 0.0, max)).unwrap(),
                NumCast::from(255).unwrap(), 
            );

            out.put_pixel(x, y, t);
        }
    }

    out
}
