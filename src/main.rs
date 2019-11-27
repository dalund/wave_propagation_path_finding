use ggez::event::{EventHandler, MouseButton};
use ggez::timer;
use ggez::{event, graphics, Context, GameResult};

const GRID_CELL_SIZE: (i32, i32) = (40, 40);
const GRID_SIZE: (i32, i32) = (15, 15);
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);

type Point2 = ggez::nalgebra::Point2<f32>;

const BORDERWIDTH: i32 = 1;
const BLUE: [f32; 4] = [0.0, 0.0, 0.5, 1.0];
const GRAY: [f32; 4] = [128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 1.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GridPosition {
    x: i32,
    y: i32,
}
impl GridPosition {
    fn new(x: i32, y: i32) -> GridPosition {
        GridPosition { x, y }
    }
}

impl From<(i32, i32)> for GridPosition {
    fn from(pos: (i32, i32)) -> GridPosition {
        GridPosition { x: pos.0, y: pos.1 }
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
    obstacle_map: Vec<GridPosition>,
    grid_mesh: Vec<graphics::Mesh>,
}
impl Game {
    fn new(ctx: &mut Context) -> GameResult<Game> {
        let mut grid_mesh = vec![];
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
            obstacle_map: vec![],
            grid_mesh,
        })
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, 60) {}
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, BLUE.into());

        for mesh in &self.grid_mesh {
            graphics::draw(ctx, mesh, graphics::DrawParam::new())?;
        }

        for obstacle in self.obstacle_map.iter() {
            let rectangle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                obstacle.into(),
                GRAY.into(),
            )?;
            graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        }

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        // convert screen x and y to grid positions
        let grid_x: i32 = x as i32 / GRID_CELL_SIZE.0;
        let grid_y: i32 = y as i32 / GRID_CELL_SIZE.1;
        if let button = MouseButton::Left {
            self.obstacle_map.push((grid_x, grid_y).into()); // TODO(david): what about remove?
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
            // add resource path
            //.add_resource_path(resource_dir)
            // And finally we attempt to build the context and create the window. If it fails, we panic with the message
            // "Failed to build ggez context"
            .build()?;

    // Next we create a new instance of our GameState struct, which implements EventHandler
    let state = &mut Game::new(ctx).unwrap();
    // And finally we actually run our game, passing in our context and state.
    event::run(ctx, events_loop, state)
}
