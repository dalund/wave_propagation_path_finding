use ggez::event::{EventHandler, MouseButton, KeyCode, KeyMods};
use ggez::timer;
use ggez::{event, graphics, Context, GameResult};
use ggez::input::keyboard;

const GRID_CELL_SIZE: (i32, i32) = (45, 45);
const GRID_SIZE: (i32, i32) = (16, 16);
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);

type Point2 = ggez::nalgebra::Point2<f32>;

const BORDERWIDTH: i32 = 1;
const BLUE: [f32; 4] = [0.0, 0.0, 0.6, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GRAY: [f32; 4] = [128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GridPosition {
    x: i32,
    y: i32,
    on: bool, // flag indicating if it in an obstacle or not
}
impl GridPosition {
    fn new(x: i32, y: i32, on: bool) -> GridPosition {
        GridPosition { x, y, on }
    }
}

impl From<(i32, i32)> for GridPosition {
    fn from(pos: (i32, i32)) -> GridPosition {
        GridPosition {
            x: pos.0,
            y: pos.1,
            on: false,
        }
    }
}

impl From<&GridPosition> for graphics::Rect {
    fn from(pos: &GridPosition) -> graphics::Rect {
        graphics::Rect::new_i32(
            (pos.x * GRID_CELL_SIZE.0) + BORDERWIDTH,
            pos.y * GRID_CELL_SIZE.1 + BORDERWIDTH,
            GRID_CELL_SIZE.0 - BORDERWIDTH,
            GRID_CELL_SIZE.1 - BORDERWIDTH,
        )
    }
}

struct Game {
    game_map: Vec<GridPosition>,
    flow_field_map_z: Vec<i32>,
    grid_mesh: Vec<graphics::Mesh>,
    starts: Vec<GridPosition>,
    end: GridPosition,
    paths: Vec<Vec<(i32, i32)>>,
}
impl Game {
    fn new(ctx: &mut Context) -> GameResult<Game> {
        let mut game_map = vec![];
        let mut grid_mesh = vec![];
        let mut flow_field_map_z = vec![];
        // initailize flow_field_map_z
        for y in 0..GRID_SIZE.1 {
            for x in 0..GRID_SIZE.0 {
                let on = x == 0 || y == 0 || x == GRID_SIZE.0 - 1 || y == GRID_SIZE.1 - 1;
                game_map.push(GridPosition::new(x as i32, y as i32, on));
                flow_field_map_z.push(0);
            }
        }

        // initailize the grid once and for all
        for x in 1..GRID_SIZE.0 {
            let line = graphics::Mesh::new_line(
                ctx,
                &[
                    Point2::new((x * GRID_CELL_SIZE.0) as f32, 0.0),
                    Point2::new(
                        (x * GRID_CELL_SIZE.0) as f32,
                        (GRID_SIZE.1 * GRID_CELL_SIZE.1) as f32,
                    ),
                ],
                2.0,
                graphics::BLACK,
            )?;
            grid_mesh.push(line);
        }
        for y in 1..GRID_SIZE.1 {
            let line = graphics::Mesh::new_line(
                ctx,
                &[
                    Point2::new(0.0, (y * GRID_CELL_SIZE.1) as f32),
                    Point2::new(
                        (GRID_SIZE.1 * GRID_CELL_SIZE.0) as f32,
                        (y * GRID_CELL_SIZE.1) as f32,
                    ),
                ],
                2.0,
                graphics::BLACK,
            )?;
            grid_mesh.push(line);
        }

        Ok(Game {
            game_map,
            grid_mesh,
            starts: vec![GridPosition::new(11, 5, false)],
            end: GridPosition::new(4, 5, false),
            flow_field_map_z,
            paths: vec![vec![]],
        })
    }

    fn calculate_path(&self,start: &GridPosition)  -> Vec<(i32, i32)> {
       let mut path: Vec<(i32, i32)> = vec![];
        path.push((start.x, start.y));
        let mut loc_x = start.x;
        let mut loc_y = start.y;
        let mut no_path = false;

        while !(loc_x == self.end.x && loc_y == self.end.y) && !no_path {
            let mut list_neightbours: Vec<(i32, i32, i32)> = vec![];

            // 4-Way Connectivity
            if loc_y - 1 >= 0 && self.flow_field_map_z[get_index(loc_x, loc_y - 1)] > 0 {
                list_neightbours.push((
                    loc_x,
                    loc_y - 1,
                    self.flow_field_map_z[get_index(loc_x, loc_y - 1)],
                ));
            }

            if (loc_x + 1) < GRID_SIZE.1
                && self.flow_field_map_z[get_index(loc_x + 1, loc_y)] > 0
            {
                list_neightbours.push((
                    loc_x + 1,
                    loc_y,
                    self.flow_field_map_z[get_index(loc_x + 1, loc_y)],
                ));
            }

            if (loc_y + 1) < GRID_SIZE.0
                && self.flow_field_map_z[get_index(loc_x, loc_y + 1)] > 0
            {
                list_neightbours.push((
                    loc_x,
                    loc_y + 1,
                    self.flow_field_map_z[get_index(loc_x, loc_y + 1)],
                ));
            }

            if (loc_x - 1) >= 0 && self.flow_field_map_z[get_index(loc_x - 1, loc_y)] > 0 {
                list_neightbours.push((
                    loc_x - 1,
                    loc_y,
                    self.flow_field_map_z[get_index(loc_x - 1, loc_y)],
                ));
            }

            // 8-Way Connectivity
            if (loc_y - 1) >= 0
                && (loc_x - 1) >= 0
                && self.flow_field_map_z[get_index(loc_x - 1, loc_y - 1)] > 0
            {
                list_neightbours.push((
                    loc_x - 1,
                    loc_y - 1,
                    self.flow_field_map_z[get_index(loc_x - 1, loc_y - 1)],
                ));
            }

            if (loc_y - 1) >= 0
                && (loc_x + 1) < GRID_SIZE.1
                && self.flow_field_map_z[get_index(loc_x + 1, loc_y - 1)] > 0
            {
                list_neightbours.push((
                    loc_x + 1,
                    loc_y - 1,
                    self.flow_field_map_z[get_index(loc_x + 1, loc_y - 1)],
                ));
            }

            if (loc_y + 1) < GRID_SIZE.0
                && (loc_x - 1) >= 0
                && self.flow_field_map_z[get_index(loc_x - 1, loc_y + 1)] > 0
            {
                list_neightbours.push((
                    loc_x - 1,
                    loc_y + 1,
                    self.flow_field_map_z[get_index(loc_x - 1, loc_y + 1)],
                ));
            }

            if (loc_y + 1) < GRID_SIZE.0
                && (loc_x + 1) < GRID_SIZE.1
                && self.flow_field_map_z[get_index(loc_x + 1, loc_y + 1)] > 0
            {
                list_neightbours.push((
                    loc_x + 1,
                    loc_y + 1,
                    self.flow_field_map_z[get_index(loc_x + 1, loc_y + 1)],
                ));
            }

            // Sprt neigbours based on height, so lowest neighbour is at front
            // of list
            list_neightbours.sort_by_key(|k| k.2);

            if list_neightbours.is_empty() {
                // Neighbour is invalid or no possible path
                no_path = true;
            } else {
                loc_x = list_neightbours[0].0;
                loc_y = list_neightbours[0].1;
                path.push((loc_x, loc_y));
            }
        }
        path
    }
}

// helper function to get index into the array
fn get_index(x: i32, y: i32) -> usize {
    (y * GRID_SIZE.0 + x) as usize
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
struct Node {
    x: i32,
    y: i32,
    d: i32,
}
impl Node {
    fn new(x: i32, y: i32, d: i32) -> Node {
        Node { x, y, d }
    }

}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, 20) {
            for tile in &self.game_map {
                if tile.x == 0
                    || tile.y == 0
                    || tile.x == (GRID_SIZE.0 - 1)
                    || tile.y == (GRID_SIZE.1 - 1)
                    || tile.on
                {
                    self.flow_field_map_z[get_index(tile.x, tile.y)] = -1;
                } else {
                    self.flow_field_map_z[get_index(tile.x, tile.y)] = 0;
                }
            }

            let mut nodes: Vec<Node> = Vec::new();
            nodes.push(Node::new(self.end.x, self.end.y, 1));

            while !nodes.is_empty() {
                let mut new_nodes: Vec<Node> = vec![];

                for node in &nodes {
                    let x = node.x;
                    let y = node.y;
                    let d = node.d;

                    self.flow_field_map_z[get_index(x, y)] = d;

                    // Check East
                    if (x + 1) < GRID_SIZE.0 && self.flow_field_map_z[get_index(x + 1, y)] == 0 {
                        new_nodes.push(Node::new(x + 1, y, d + 1));
                    }

                    // Check West
                    if (x - 1) >= 0 && self.flow_field_map_z[get_index(x - 1, y)] == 0 {
                        new_nodes.push(Node::new(x - 1, y, d + 1));
                    }

                    // Check South
                    if (y + 1) < GRID_SIZE.1 && self.flow_field_map_z[get_index(x, y + 1)] == 0 {
                        new_nodes.push(Node::new(x, y + 1, d + 1));
                    }

                    // Check North
                    if (y - 1) >= 0 && self.flow_field_map_z[get_index(x, y - 1)] == 0 {
                        new_nodes.push(Node::new(x, y - 1, d + 1));
                    }
                }

                new_nodes.sort();
                new_nodes.dedup(); // remove duplicates. Have to sort first.

                nodes.clear();
                nodes = new_nodes.drain(0..).collect();
            }

            self.paths.clear();
            for (i, start) in self.starts.iter().enumerate() {
                let path = self.calculate_path(start);

                self.paths.push(path.to_vec());
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, BLUE.into());

        // draw grid
        for mesh in &self.grid_mesh {
            graphics::draw(ctx, mesh, graphics::DrawParam::new())?;
        }

        // draw tile
        for tile in self.game_map.iter() {
            if tile.on {
                let rectangle = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    tile.into(),
                    GRAY.into(),
                )?;
                graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
            }
        }


        // draw start and end
        for start in &self.starts {
            let rectangle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                start.into(),
                GREEN.into(),
            )?;
            graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;


        }
        let end = &self.end;
        let rectangle =
            graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), end.into(), RED.into())?;
        graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;


        // draw path
        for path in &self.paths {
            let mut first_point = true;
            let mut o_x = 0;
            let mut o_y = 0;
            for t in path {
                if first_point {
                    o_x = t.0;
                    o_y = t.1;
                    first_point = false;
                } else {
                    // draw connecting line
                    let line = graphics::Mesh::new_line(
                        ctx,
                        &[to_screen_circle(o_x, o_y), to_screen_circle(t.0, t.1)],
                        2.0,
                        YELLOW.into(),
                    )?;

                    o_x = t.0;
                    o_y = t.1;

                    let circle = graphics::Mesh::new_circle(
                        ctx,
                        graphics::DrawMode::fill(),
                        to_screen_circle(o_x, o_y),
                        14.0,
                        0.2,
                        YELLOW.into(),
                    )?;
                    graphics::draw(ctx, &line, graphics::DrawParam::default())?;
                    graphics::draw(ctx, &circle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
                }
            }
        }

        // Note(david): uncomment to see all distances
        // for y in 0..GRID_SIZE.1 {
        //     for x in 0..GRID_SIZE.0 {

        //         // draw distace string
        //         let text_fragment = graphics::TextFragment::new(format!("{}", self.flow_field_map_z[ get_index(x, y)])) .color(graphics::BLACK);
        //         let text = graphics::Text::new(text_fragment);
        //         graphics::queue_text(ctx, &text, to_screen(x, y), None);

        //     }
        // }
        // graphics::draw_queued_text(ctx, graphics::DrawParam::default(), None, graphics::FilterMode::Linear)?;

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        // convert screen x and y to grid positions
        let grid_x: i32 = x as i32 / GRID_CELL_SIZE.0;
        let grid_y: i32 = y as i32 / GRID_CELL_SIZE.1;
        let newlen = self.starts.len() - 1;
        match button {
            MouseButton::Left => {
                // get obstacle selected
                let ob = &mut self.game_map[(grid_y * GRID_SIZE.0 + grid_x) as usize];
                ob.on = !ob.on;
            }
            MouseButton::Right => {
                if keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
                    self.starts.push(GridPosition::new(grid_x, grid_y, false));
                } else {
                    self.starts[newlen] = GridPosition::new(grid_x, grid_y, false);
                }
            }
            _ => {}
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::Q => {
                // remove last added start position
                self.starts.pop();
            }
            _ => {
            }
        }
    }
}

fn main() -> GameResult {
    let (ctx, events_loop) =
        &mut ggez::ContextBuilder::new("Path plannig - wave propagation", "David Lundell")
            // Next we set up the window. This title will be displayed in the title bar of the window.
            .window_setup(
                ggez::conf::WindowSetup::default().title("Path plannig - wave propagation!"),
            )
            // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
            .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
            // And finally we attempt to build the context and create the window. If it fails, we panic with the message
            // "Failed to build ggez context"
            .build()?;

    // Next we create a new instance of our GameState struct, which implements EventHandler
    let state = &mut Game::new(ctx).unwrap();
    // And finally we actually run our game, passing in our context and state.
    event::run(ctx, events_loop, state)
}

fn to_screen_circle(x: i32, y: i32) -> Point2 {
    Point2::new(
        (x * GRID_CELL_SIZE.0 + (GRID_CELL_SIZE.0 / 2)) as f32,
        (y * GRID_CELL_SIZE.1 + (GRID_CELL_SIZE.1 / 2)) as f32,
    )
}
