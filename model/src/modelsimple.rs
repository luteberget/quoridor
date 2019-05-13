

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Orientation {
    Horizontal, Vertical
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Position {
    x: i64,
    y: i64
}

pub struct Board {
    player :usize,
    positions :[Position;2],
    walls :Vec<(Orientation,Position)>
        // a wall is an orientation and a coordinate.
        // for horizontal walls:
        // _a_b_
        //  c d  we give the top left (lowest) coordinate, a.
        //
        // for vertical walls:
        //  a|b
        //  c|d  we give the top left (lowest) coordinate, a.
        //
        //  So both x and y coordinates can be in [1,8]. 
        //  X=9 would place the vertical wall outside the board, horizontal would stick out.
        //  Y=9 would place the horizontal wall below the board, vertical would stick out.
        //
        //  checking whether two coordinates between 4-connected positions 
        //  are separated by a wall, amounts to checking:
        //   i. (x,y)-(x+1,y)  -- check vertical walls 
        //      at position x or x-1, y=y.
        //  ii. (x,y)-(x,y+1)  -- check horizontal walls
        //      at position x=x, y= y or y-1.
}

impl Default for Board {
    fn default() -> Board {
        // Starting positions for players; no walls.
        Board {
            player: 0,
            positions: [
                Position { x: 5, y: 9 },
                Position { x: 5, y: 1 },
            ],
            walls: Vec::new(),
        }
    }
}

pub enum Move {
    PawnTo(Position),
    WallAt(Orientation,Position),
}

impl Board {
    pub fn integrate(&mut self, mv :Move) -> Result<(),()> {
        match mv {
            Move::PawnTo(pos) => {
                if !self.is_valid_move(&pos) { return Err(()); }
                self.positions[self.player] = pos;
            },
            Move::WallAt(ori,pos) => {
                if !self.can_add_wall(ori,pos) { return Err(()); }
                self.walls.push((ori,pos));
                // TODO sort?
            }
        }

        self.player = 1 - self.player;
        Ok(())
    }

    pub fn is_valid_move(&self, pos :&Position) -> bool {
        if !in_bounds(pos) { return false; }
        if is_neighbor(&self.positions[self.player], pos) {
            if !self.is_empty(pos) { return false; }
            if self.wall_between(&self.positions[self.player], pos) { return false; }
            true
        } else {
            self.is_valid_jump(pos)
        }
    }

    pub fn is_empty(&self, pos :&Position) -> bool {
        self.positions[0] != *pos && self.positions[1] != *pos
    }

    /// Checks whether two points, which are 4-connected neighbors,
    /// are separated by a wall. Return true if they are,
    /// and false if they are not separated by a wall, or if they are not
    /// 4-connected neighbors.
    pub fn wall_between(&self, a :&Position, b :&Position) -> bool {
        if (b.x-a.x).abs() == 1 && b.y-a.y == 0 {
            // check vertical walls
            for (ori,pos) in &self.walls {
                if let Orientation::Vertical = ori {
                    let left = b.x.min(a.x);
                    if left == pos.x && (pos.y == b.y || pos.y +1 == b.y) {
                        return true;
                    }
                }
            }
        } else if (b.y-a.y).abs() == 1 && b.x-a.x == 0 {
            // check horizontal walls
            for (ori,pos) in &self.walls {
                if let Orientation::Horizontal = ori {
                    let top = b.y.min(a.y);
                    if top == pos.y && (pos.x == b.x || pos.x +1 == b.x) {
                        return true;
                    }
                }
            }
        } else {
            panic!();
        }
        false
    }

    pub fn can_add_wall(&self, ori :Orientation, pos :Position) -> bool {
        for (o,p) in &self.walls {
            if wall_conflicts(&ori,&pos,o,p) {
                return false;
            }
        }

        if !self.goal_reachable(ori,pos) {
            return false;
        }

        true
    }

    pub fn is_valid_jump(&self, pos :&Position) -> bool {
        let start = self.positions[self.player];
        let end   = *pos;

        let horizontal_jump = (end.x - start.x).abs() == 2 && end.y == start.y;
        let vertical_jump   = (end.y - start.y).abs() == 2 && end.x == start.x;
        let diagonal_jump   = (end.x - start.x).abs() == 1 && (end.y - start.y).abs() == 1;

        let other_player_pos = self.positions[1 - self.player];

        if horizontal_jump || vertical_jump {
            let middle_pos = Position { x: (start.x + end.x)/2, y: (start.y + end.y)/2 };

           middle_pos == other_player_pos && 
               !self.wall_between(&start,&middle_pos) && 
               !self.wall_between(&middle_pos,&end)

        } else if diagonal_jump {
            let other_side = Position { x: other_player_pos.x + (other_player_pos.x - start.x),
                                        y: other_player_pos.y + (other_player_pos.y - start.y)};
            self.wall_between(&other_player_pos, &other_side) && 
                !self.wall_between(&start, &other_player_pos) && 
                !self.wall_between(&other_player_pos, &end)

        } else {
            false
        }
    }

    pub fn goal_reachable(&self, ori :Orientation, pos :Position) -> bool {
        let mut horizontal_walls = 0u64;
        let mut vertical_walls = 0u64;

        bitset_add_wall(&mut horizontal_walls, &mut vertical_walls, &ori, &pos);
        for (ori,pos) in &self.walls {
            bitset_add_wall(&mut horizontal_walls, &mut vertical_walls, &ori, &pos);
        }

        // TODO store bit set directly in Board instead of converting 
        // vector of walls (not needed information when game is progressing forward).


        // player 1 must be able to reach top
        // and player 2 must be able to reach bootom
        let p1 = goal_reachable(horizontal_walls,
                       vertical_walls,
                       self.positions[0], 1);
        let p2 = goal_reachable(horizontal_walls,
                       vertical_walls,
                       self.positions[1], 9);
        p1 && p2
    }
}

fn bitset_add_wall(horizontal_walls :&mut u64, vertical_walls :&mut u64, 
                   ori :&Orientation, pos :&Position) {
    use bit_field::BitField;
    match ori {
        Orientation::Horizontal => {
            horizontal_walls.set_bit(encode8(pos.x-1,pos.y-1), true);
        },
        Orientation::Vertical => {
            vertical_walls.set_bit(encode8(pos.x-1,pos.y-1), true);
        },
    }
}

fn encode9(x :i64, y :i64) -> usize {
    (x+9*y) as usize
}
fn encode8(x :i64, y :i64) -> usize {
    (x+8*y) as usize
}

pub fn goal_reachable(horizontal_walls: u64,
                      vertical_walls: u64, 
                      pos :Position, 
                      goal_row :i64) -> bool {
    use bit_field::BitField;
    use disjoint_sets::UnionFind;
    let mut uf = UnionFind::new(9*9);


    // TODO: It could be that finding connectivity
    // in the dual graph is faster, becuse much of the
    // size of the state space is going to be in placing walls
    // when there is few walls and little chance to break connnectivity.
    // In that case, iterating only over the set bits of the 
    // walls bitsets and connecting nodes in the dual graph might be faster.

    for x in 0..=8 {
        for y in 0..=8 {
            let this_node = encode9(x,y);
            if x < 8  {
                // go right
                if !(y < 8 && vertical_walls.get_bit(encode8(x,y))) &&
                   !(y > 0 && vertical_walls.get_bit(encode8(x,y-1))) {
                       uf.union(this_node,encode9(x+1,y));
                   }
            }
            if x > 0  {
                // go left 
                if !(y < 8 && vertical_walls.get_bit(encode8(x-1,y))) &&
                   !(y > 0 && vertical_walls.get_bit(encode8(x-1,y-1))) {
                       uf.union(this_node,encode9(x-1,y));
                   }
            }

            if y < 8  {
                // go down
                if !(x < 8 && horizontal_walls.get_bit(encode8(x,y))) &&
                   !(x > 0 && horizontal_walls.get_bit(encode8(x-1,y))) {
                       uf.union(this_node,encode9(x,y+1));
                   }
            }
            if y > 0  {
                // go up
                if !(x < 8 && horizontal_walls.get_bit(encode8(x,y-1))) &&
                   !(x > 0 && horizontal_walls.get_bit(encode8(x-1,y-1))) {
                       uf.union(this_node,encode9(x,y-1));
                   }
            }
        }
    }

    let this_value = uf.find(encode9(pos.x-1,pos.y-1));
    for x in 0..=8 {
        if this_value == uf.find(encode9(x,goal_row-1)) {
            return true;
        }
    }

    return false;
}

fn wall_conflicts(oa :&Orientation, pa :&Position, ob :&Orientation, pb :&Position) -> bool {
    if oa == ob {
        match oa {
            Orientation::Horizontal => {
                !(pa.y == pb.y && (pa.x == pb.x || pa.x + 1 == pb.x || pb.x + 1 == pa.x))
            },
            Orientation::Vertical => {
                !(pa.x == pb.x && (pa.y == pb.y || pa.y + 1 == pb.y || pb.y + 1 == pa.y))
            },
        }
    } else {
        if let Orientation::Vertical = oa {
            // A is vertical, B is horizontal
            !(pa == pb)
        } else {
            wall_conflicts(ob,pb,oa,pa)
        }
    }
}

fn in_bounds(pos :&Position) -> bool {
    pos.x > 0 && pos.x <= 9 && pos.y > 0 && pos.y <= 9
}

fn is_neighbor(a :&Position, b :&Position) -> bool {
    if a.x == b.x {
        a.y + 1 == b.y || b.y + 1 == a.y
    } else if a.y == b.y {
        a.x + 1 == b.x || b.x + 1 == a.x
    } else {
        false
    }
}


