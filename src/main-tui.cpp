#include "controller.hpp"
#include <boost/asio/ip/resolver_base.hpp>
#include <boost/beast/core.hpp>
#include <boost/beast/websocket.hpp>
#include <boost/beast/websocket/ssl.hpp>
#include <boost/asio/connect.hpp>
#include <boost/asio/ip/tcp.hpp>
#include <boost/asio/ssl/stream.hpp>
#include <boost/json.hpp>
#include <boost/json/kind.hpp>
#include <boost/json/object.hpp>
#include <boost/json/parse.hpp>
#include <boost/json/value.hpp>
#include <iostream>
#include <ncurses.h>
#include <optional>
#include <unordered_map>

using tcp = boost::asio::ip::tcp;

const std::array<std::string, 11> pixels = {
    "┌───────────┐",
    "│⏻          │",
    "│           │",
    "│        │",
    "│󰝟      │",
    "│        │",
    "│           │",
    "│        │",
    "│           │",
    "│         │",
    "└───────────┘"
};

struct Pos {
    int x;
    int y;

    bool operator==(const Pos &p) const {
        return p.x == x && p.y == y;
    }
};

template <>
struct std::hash<Pos> {
    std::size_t operator()(const Pos& p) const noexcept {
        return std::hash<int>()(p.x) ^ (std::hash<int>()(p.y) << 1);
    }
};

std::unordered_map<Pos, Controller::Key> postition_map = {
    {{1, 2}, Controller::Key::POWER},
    {{1, 4}, Controller::Key::VOL_UP},
    {{1, 5}, Controller::Key::MUTE},
    {{1, 6}, Controller::Key::VOL_DOWN},
    {{6, 4}, Controller::Key::UP},
    {{7, 4}, Controller::Key::UP},
    {{6, 5}, Controller::Key::ENTER},
    {{7, 5}, Controller::Key::ENTER},
    {{4, 5}, Controller::Key::LEFT},
    {{5, 5}, Controller::Key::LEFT},
    {{8, 5}, Controller::Key::RIGHT},
    {{9, 5}, Controller::Key::RIGHT},
    {{6, 6}, Controller::Key::DOWN},
    {{7, 6}, Controller::Key::DOWN},
    {{11, 4}, Controller::Key::CH_UP},
    {{11, 6}, Controller::Key::CH_DOWN},
    {{4, 8}, Controller::Key::BACK},
    {{6, 8}, Controller::Key::PLAY},
    {{8, 8}, Controller::Key::PAUSE},
    {{4, 10}, Controller::Key::HOME},
    {{5, 10}, Controller::Key::HOME},
    {{8, 10}, Controller::Key::MENU},
    {{9, 10}, Controller::Key::MENU},
};
//
std::optional<Controller::Key> parse_key(int key) {
    MEVENT event;
    if (key == KEY_UP) {
        return Controller::Key::UP;
    } else if (key == KEY_DOWN) {
        return Controller::Key::DOWN;
    } else if (key == KEY_LEFT) {
        return Controller::Key::LEFT;
    } else if (key == KEY_RIGHT) {
        return Controller::Key::RIGHT;
    } else if (key == 10) {
        return Controller::Key::ENTER;
    } else if (key == 27) {
        return Controller::Key::BACK;
    } else if (key == 'h') {
        return Controller::Key::HOME;
    } else if (key == 'j') {
        return Controller::Key::CH_DOWN;
    } else if (key == 'k') {
        return Controller::Key::CH_UP;
    } else if (key == 'c') {
        return Controller::Key::MENU;
    } else if (key == 'm') {
        return Controller::Key::MUTE;
    } else if (key == '+' || key == '=') {
        return Controller::Key::VOL_UP;
    } else if (key == '-' || key == '_') {
        return Controller::Key::VOL_DOWN;
    } else if (key == KEY_MOUSE && getmouse(&event) == OK) {
        Pos p{event.x, event.y };
        if (postition_map.contains(p)) {
            return postition_map.at(p);
        }
    }
    // else {
    //     mvprintw(13, 0, "Key pressed %d\n", key);
    // }
    return {};
}

void print_controller() {
    mvprintw(1, 0, "%s\n", pixels[0].c_str());
    for (int i = 1; i < pixels.size(); i++) {
        printw("%s\n", pixels[i].c_str());
    }
}

void setup_ncurses() {

}

int main(int argc, char *argv[]) {
    setlocale(LC_CTYPE, "");
    initscr();
    cbreak();
    noecho();
    curs_set(0);
    keypad(stdscr, TRUE);
    mousemask(ALL_MOUSE_EVENTS | REPORT_MOUSE_POSITION, NULL);
    
    int rows = 12;
    int cols = 13;
    printw("Press q to quit...\n");
    try {
        boost::asio::io_context ioc;
        Controller c {ioc};

        while (true) {
            if (!c.is_open()) {
                c.reconnect();
                if (!c.is_open()) {
                    printw("Connection with TV lost, closing...");
                    break;
                }
            }
            print_controller();
            int key = getch();
            if (key == 'q') break;
            auto cmd = parse_key(key);
            if (cmd) {
                c.send_key(*cmd);
            }
        }
    } 
    catch (std::exception const& e) {
        std::cerr << "Fatal Execution Error: " << e.what() << std::endl;
        endwin();
        return EXIT_FAILURE;
    }
    endwin();
    return EXIT_SUCCESS;
}
