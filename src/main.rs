use ggez::{graphics, event, GameResult, Context};
use ggez::event::{EventHandler, MouseButton};
use ggez::timer;

const GRID_CELL_SIZE: (i32, i32) = (40, 40);
const GRID_SIZE: (i32, i32) = (15, 15);
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 *GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 *GRID_CELL_SIZE.1 as f32
);


const BORDERWIDTH : f32 = 1.0;
const BLUE: [f32; 4] = [0.0, 0.0, 0.5, 1.0];
const GRAY: [f32; 4] = [128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 1.0];


struct Game {
    obstacle_map: [bool; GRID_SIZE.0 as usize*GRID_SIZE.1 as usize],
}
impl Game {
    fn new() -> GameResult<Game> {

        Ok(Game {
            obstacle_map: [false; GRID_SIZE.0 as usize*GRID_SIZE.1 as usize],
        })
    }

    fn get_cell_index(&self, x: i32, y: i32) -> usize {
        (y*GRID_SIZE.0 + x) as usize
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, 60) {
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        for y in 0..GRID_SIZE.1 {
            for x in 0..GRID_SIZE.0 {
                let rectangle = graphics::Rect::new(
                    (x * GRID_CELL_SIZE.0) as f32 + BORDERWIDTH,
                    ( y * GRID_CELL_SIZE.1 ) as f32 + BORDERWIDTH,
                    GRID_CELL_SIZE.0 as f32 - BORDERWIDTH,
                    GRID_CELL_SIZE.1 as f32 - BORDERWIDTH
                );
                let rect = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rectangle, BLUE.into())?;
                graphics::draw(ctx, &rect, graphics::DrawParam::default())?;

                if self.obstacle_map[self.get_cell_index(x, y)] {
                    let obstacle_rect = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rectangle, GRAY.into())?;
                    graphics::draw(ctx, &obstacle_rect, graphics::DrawParam::default())?;
                }
            }
        }

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }


    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32,  y: f32) {
        // convert screen x and y to grid positions
        let grid_x: i32 = x as i32 / GRID_CELL_SIZE.0;
        let grid_y: i32 = y as i32 / GRID_CELL_SIZE.1;
        match button {
            MouseButton::Left => {
                self.obstacle_map[self.get_cell_index(grid_x, grid_y)] = !self.obstacle_map[self.get_cell_index(grid_x, grid_y)]
            }
            _ => {}
        }

    }
}

fn main() -> GameResult {
    let (ctx, events_loop) = &mut ggez::ContextBuilder::new("Path plannig - wave propagation", "David Lundell")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title("Path plannig - wave propagation!"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        // add resource path
        //.add_resource_path(resource_dir)
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .build()?;

    // Next we create a new instance of our GameState struct, which implements EventHandler
    let state = &mut Game::new().unwrap();
    // And finally we actually run our game, passing in our context and state.
    event::run(ctx, events_loop, state)
}
