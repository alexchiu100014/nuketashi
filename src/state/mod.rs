//! ゲームの場面に相当するステートを担う
//! モジュール．

use piston_window::*;
use std::collections::VecDeque;

pub mod hello;

/// ゲームの状態．一般的な"シーン"に対応するトレイト．
/// 描画ルーチンと更新ルーチンを担う．
pub trait State {
    /// ↓これが毎フレーム呼び出される。
    /// ```
    /// window.draw_2d(&evt, |c, g, _d| {
    ///     scene.draw((f64::from(w[0]), f64::from(w[1])), c, g)
    /// ```
    fn draw(&mut self, viewport: (f64, f64), context: Context, graf: &mut G2d) -> Option<()>;

    /// 毎フレーム`StateManager`によって呼び出される。`event: &mut EventPool` に
    /// 1つ前のフレームから現在までにあったイベント（クリックとかキー押下とか）のリストが入っている。
    /// ### Examples
    /// ```
    /// fn update(&mut self, event: &mut EventPool, _: f64) -> Option<()> {
    ///     // 画面をクリックしたらGameシーンへ遷移
    ///     while let Some(evt) = event.poll_event() {
    ///         if evt == crate::state::GameEvent::MouseDown {
    ///             event.set_next_state(crate::state::game::Game::new());
    ///         }
    ///     }
    ///     Some(())
    /// }
    /// ```
    fn update(&mut self, event: &mut EventPool, delta_time: f64) -> Option<()>;

    fn font_glyphs(&mut self) -> Option<&mut Glyphs> {
        None
    }
}

/// ステートを保持し，Stateに描画ルーチンと更新ルーチンを発行する．
/// イベントキューを保有する．
#[derive(Default)]
pub struct StateManager<'t> {
    // ゲームのシーンを動的に保持
    scene: Option<Box<dyn State + 't>>,
    // イベントキュー
    events: EventPool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameEvent {
    KeyDown(i32),
    KeyUp(i32),
    MouseDown,
    MouseUp,
    MouseMove,
}

/// イベントキュー．カーソルの位置と次のステートを保有する．
/// eventsフィールドに [KeyDown(32), MouseDown, MouseUp, ...] みたいのが入る。
#[derive(Default)]
pub struct EventPool {
    // カーソル位置などのメモ
    cursor_x: f64, // 0 ~ 1
    cursor_y: f64, // 0 ~ 1
    events: VecDeque<GameEvent>,
    next_state: Option<Box<dyn State>>,
}

impl EventPool {
    pub fn cursor(&self) -> (f64, f64) {
        (self.cursor_x, self.cursor_y)
    }

    /// `self.events.push_back(evt)` する。
    pub fn push_event(&mut self, evt: GameEvent) {
        self.events.push_back(evt)
    }

    /// `self.event.pop_front()` を返す。
    pub fn poll_event(&mut self) -> Option<GameEvent> {
        self.events.pop_front()
    }

    /// 次のシーンを設定する
    pub fn set_next_state<T>(&mut self, state: T)
    where
        T: State + 'static,
    {
        self.next_state = Some(Box::new(state));
    }

    /// 次のシーンを削除と同時に取り出す。シーンマネージャーから呼ばれる。
    pub fn pull_next_state(&mut self) -> Option<Box<dyn State>> {
        std::mem::replace(&mut self.next_state, None)
    }

    /// 現在溜まっているイベントをすべて捨てる。
    pub fn clear(&mut self) {
        self.events.clear()
    }
}

impl Iterator for EventPool {
    type Item = GameEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.poll_event()
    }
}

impl<'t> StateManager<'t> {
    pub fn new() -> StateManager<'t> {
        Self::default()
    }

    pub fn set_state<T>(&mut self, scene: T)
    where
        T: State + 't,
    {
        self.scene = Some(Box::new(scene));
    }

    pub fn handle_event<T>(&mut self, window: &mut PistonWindow<T>, evt: Event)
    where
        T: OpenGLWindow,
    {
        // 次のシーンに切り替えを行う（必要な場合）
        if self.events.next_state.is_some() {
            self.scene = self.events.pull_next_state();
        }

        // シーンがある場合のみ更新する．
        if self.scene.is_none() {
            return;
        }

        let events = &mut self.events;

        // ウィンドウからイベントを取得して，キューに追加
        // カーソル位置の更新
        if let Some([x, y]) = evt.mouse_cursor_args() {
            events.cursor_x = x / window.draw_size().width;
            events.cursor_y = y / window.draw_size().height;

            events.push_event(GameEvent::MouseMove);
        }

        // マウス・キーボードのイベント処理
        if let Some(btn) = evt.press_args() {
            match btn {
                Button::Keyboard(key) => events.push_event(GameEvent::KeyDown(key.code())),
                Button::Mouse(_) => {
                    events.push_event(GameEvent::MouseDown);
                }
                _ => {}
            }
        }
        if let Some(btn) = evt.release_args() {
            match btn {
                Button::Keyboard(key) => events.push_event(GameEvent::KeyUp(key.code())),
                Button::Mouse(_) => {
                    events.push_event(GameEvent::MouseUp);
                }
                _ => {}
            }
        }

        // シーンに問い合わせてゲームのアップデート
        //（ゲームの更新はメインスレッドから行われることを前提とする）
        let scene = self.scene.as_mut().unwrap();

        if let Some(args) = evt.update_args() {
            let delta_time = args.dt;
            scene.update(events, delta_time).unwrap();
        }

        if let Some(v) = evt.render_args() {
            let w = v.draw_size;

            window.draw_2d(&evt, |c, g, d| {
                scene
                    .draw((f64::from(w[0]), f64::from(w[1])), c, g)
                    .unwrap();

                if let Some(glyphs) = scene.font_glyphs() {
                    glyphs.factory.encoder.flush(d);
                }
            });
        }
    }
}
