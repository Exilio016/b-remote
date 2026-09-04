#include "controller.hpp"
#include <boost/asio/io_context.hpp>
#include <exception>
#include <raylib-cpp.hpp>
#include <array>
#include <raylib.h>
#include <string>
#include <thread>

constexpr int GRID_COLS = 7;
constexpr int GRID_ROWS = 9;
constexpr int W_HEIGHT = 350;
constexpr int W_WIDTH = 200;

raylib::Vector4 find_grid_position(int row, int col) {
    int cell_width = W_WIDTH / GRID_COLS;
    int cell_height = W_HEIGHT / GRID_ROWS;
    raylib::Vector4 v { static_cast<float>(col * cell_width), static_cast<float>(row * cell_height),
        static_cast<float>(cell_width), static_cast<float>(cell_height) };

    return v;
}

constexpr raylib::Rectangle TEX_SIZE = { 0.0, 0.0, 24.0, 24.0 };

struct Button {
    raylib::Vector4 pos;
    raylib::Texture tex;
    Controller::Key key;
    int clicked_frames;
    int keycode;

    Button(int row, int col, Controller::Key key, int keycode, std::string texture) {
        if (!texture.empty()) {
            texture = std::string(GetApplicationDirectory()) + texture;
            tex.Load(texture);
        }
        pos = find_grid_position(row, col);
        this->key = key;
        clicked_frames = 0;
        this->keycode = keycode;
    }
    void draw() const {
        raylib::Rectangle rect { pos };
        if (clicked_frames > 0) {
            rect.DrawRounded(0.8, 16, raylib::Color::LightGray());
        }
        else {
            rect.Draw(raylib::Color::RayWhite());
        }
        if (tex.IsValid()) {
            tex.Draw(TEX_SIZE, pos);
        }
    }

    void update(Controller &c, int k) {
        if (clicked_frames > 0 ) {
            clicked_frames--;
        }
        if (clicked() || k == keycode) {
            c.send_key(key);
            clicked_frames = 15;
        }
    }

    bool clicked() const {
        if (!raylib::Mouse::IsButtonPressed(MOUSE_BUTTON_LEFT) && 
            !(clicked_frames == 1 && raylib::Mouse::IsButtonDown(MOUSE_BUTTON_LEFT))) {
            return false;
        }
        auto mouse_pos = raylib::Mouse::GetPosition();
        return mouse_pos.x > pos.x && mouse_pos.x < (pos.x + pos.z)
            && mouse_pos.y > pos.y && mouse_pos.y < (pos.y + pos.w);
    }
};

int main() {
    raylib::Window w {W_WIDTH, W_HEIGHT, "B Controller"};
    w.SetTargetFPS(60);
    w.SetExitKey(KEY_NULL);

    boost::asio::io_context ioc { 4 };
    auto c = std::make_shared<Controller> (ioc);
    c->connect();
    std::jthread thread {[&ioc] {ioc.run();}};

    std::array<Button, 17> buttons = {
        Button {0, 0, Controller::Key::POWER,   KEY_P, std::string("assets/circle-power.png")},
        Button {2, 0, Controller::Key::VOL_UP,  KEY_EQUAL, std::string("assets/arrow-up.png")},
        Button {3, 0, Controller::Key::MUTE,    KEY_M, std::string("assets/volume-x.png")},
        Button {4, 0, Controller::Key::VOL_DOWN,KEY_MINUS, std::string("assets/arrow-down.png")},
        Button {2, 6, Controller::Key::CH_UP,   KEY_K, std::string("assets/arrow-up.png")},
        Button {3, 6, Controller::Key::CH_LIST,    KEY_G, std::string("assets/tv.png")},
        Button {4, 6, Controller::Key::CH_DOWN, KEY_J, std::string("assets/arrow-down.png")},
        Button {2, 3, Controller::Key::UP,      KEY_UP, std::string("assets/circle-arrow-up.png")},
        Button {3, 2, Controller::Key::LEFT,    KEY_LEFT, std::string("assets/circle-arrow-left.png")},
        Button {3, 3, Controller::Key::ENTER,   KEY_ENTER, std::string("assets/circle.png")},
        Button {3, 4, Controller::Key::RIGHT,   KEY_RIGHT, std::string("assets/circle-arrow-right.png")},
        Button {4, 3, Controller::Key::DOWN,    KEY_DOWN, std::string("assets/circle-arrow-down.png")},
        Button {6, 2, Controller::Key::BACK,    KEY_ESCAPE, std::string("assets/undo-2.png")},
        Button {6, 3, Controller::Key::PLAY,    KEY_SPACE, std::string("assets/play.png")},
        Button {6, 4, Controller::Key::PAUSE,   KEY_Z, std::string("assets/pause.png")},
        Button {8, 2, Controller::Key::HOME,    KEY_H, std::string("assets/house.png")},
        Button {8, 4, Controller::Key::MENU,    KEY_S, std::string("assets/settings.png")},
    };

    while (!w.ShouldClose() && c->is_open()) {
        try {
            int k = GetKeyPressed();
            w.BeginDrawing();
            w.ClearBackground(raylib::Color::RayWhite());
            for (auto &b : buttons) {
                b.update(*c, k);
                b.draw();
            }
            w.EndDrawing();
        } catch (std::exception const &e) {
            TraceLog(LOG_ERROR, "Exception occurred %s", e.what());
        }
    }
    c->close();
    return 0;
}
