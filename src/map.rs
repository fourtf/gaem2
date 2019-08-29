use rect::Rect;

pub struct Map {
    pub blocks: Vec<Vec<u8>>,
}

impl Map {
    pub fn new(blocks: Vec<Vec<u8>>) -> Map {
        return Map { blocks: blocks };
    }

    pub fn move_item(&self, rect: &mut Rect, dx: f64, dy: f64, time_passed: f64) -> Collision {
        let mut collision = Collision::default();

        // y position
        if dy != 0.0 {
            rect.y += dy * time_passed;

            if dy > 0.0 {
                for x in rect.x as i64..(rect.right() + 0.9999) as i64 {
                    if self.get_i(x, rect.bottom() as i64) == 1 {
                        rect.move_bottom(rect.bottom() as i64 as f64);
                        collision.bottom = true;
                        break;
                    }
                }
            } else {
                for x in rect.x as i64..(rect.right() + 0.999) as i64 {
                    if self.get_i(x, rect.y.floor() as i64) == 1 {
                        rect.y = (rect.y.floor() as i64 + 1) as f64;
                        collision.top = true;
                        break;
                    }
                }
            }
        }

        // x position
        if dx != 0.0 {
            rect.x += dx * time_passed;

            if dx > 0.0 {
                for y in rect.y as i64..(rect.bottom() + 0.999) as i64 {
                    if self.get_i(rect.right() as i64, y) == 1 {
                        rect.move_right(rect.right() as i64 as f64);
                        collision.right = true;
                        break;
                    }
                }
            } else {
                for y in rect.y as i64..(rect.bottom() + 0.999) as i64 {
                    if self.get_i(rect.x.floor() as i64, y) == 1 {
                        rect.x = (rect.x.floor() as i64 + 1) as f64;
                        collision.left = true;
                        break;
                    }
                }
            }
        }

        collision
    }

    fn get_i(&self, x: i64, y: i64) -> u8 {
        //println!("getting {} {}", x, y);
        if x < 0 || y < 0 || y >= self.blocks.len() as i64 {
            1u8
        } else {
            let line = &self.blocks[y as usize];

            if x >= line.len() as i64 {
                1u8
            } else {
                line[x as usize]
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct Collision {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
}

impl Collision {
    pub fn is_on_floor(&self) -> bool {
        return self.bottom;
    }
}
