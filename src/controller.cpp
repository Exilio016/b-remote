#include "controller.hpp"
#include <boost/asio/io_context.hpp>
#include <boost/beast/core.hpp>
#include <boost/json.hpp>
#include <cstdlib>
#include <filesystem>
#include <fstream>

#ifdef DEBUG_SSL_CONNECTION
#include <openssl/ssl.h>
#endif

constexpr std::string TV_HOST = "Samsung.local";
constexpr std::string TV_PORT = "8002";
constexpr std::string_view APP_NAME_B64 = "Yi1jb250cm9sbGVy";

namespace beast = boost::beast;
namespace json = boost::json;

std::filesystem::path get_token_path() {
    auto config = std::getenv("XDG_CONFIG_HOME");
    std::filesystem::path path;
    if (config && *config) {
        path.append(config);
    }
    else {
        auto home = std::getenv("HOME");
        if (home && *home) {
            path.append(home);
            path.append(".config");
        }
        else {
            path.append(".");
        }
    }
    path.append("b-controller.token.txt");
    return path;
}

std::optional<std::string> get_token_from_config() {
    auto path = get_token_path();
    std::ifstream file;
    file.open(path);
    if (file.is_open()) {
        std::string token;
        file >> token;
        return token;
    }

    return {};
}

void save_token_to_config(std::string_view token) {
    auto path = get_token_path();
    std::ofstream file;
    file.open(path);
    if (file.is_open()) {
        file << token;
    }
}

Controller::Controller(boost::asio::io_context &ioc): m_ioc(ioc),
    m_ssl_ctx(ssl::context::tlsv12_client), m_ws(ioc, m_ssl_ctx) {
    m_token = get_token_from_config();
}

bool Controller::is_open() {
    return m_ws.is_open();
}

void Controller::read_event() {
    if (m_ws.is_open()) {
        beast::flat_buffer buffer;
        m_ws.read(buffer);
        auto txt = beast::buffers_to_string(buffer.data());
        buffer.clear();

        json::value json = json::parse(txt);
        if (!json.is_object()) {
            return;
        }
        json::object obj = json.as_object();
        if (!obj.contains("event")) {
            return;
        }
        auto event = obj.at("event").as_string();
        if (event != "ms.channel.connect") {
            return;
        }
        auto data = obj.at("data").as_object();
        if (data.contains("token")) {
            auto token = data.at("token").as_string();
            save_token_to_config(token);
            m_token = token;
        }
    }
}

#ifdef DEBUG_SSL_CONNECTION
std::mutex keylog_mutex;

void ssl_keylog_callback(const SSL* /*ssl*/, const char* line)
{
    std::lock_guard<std::mutex> lock(keylog_mutex);

    std::ofstream file("tls-keys.log", std::ios::app);
    file << line << '\n';
}
#endif

void Controller::connect() {
    tcp::resolver resolver{m_ioc};
    auto const endpoint = resolver.resolve(TV_HOST, TV_PORT, tcp::resolver::flags());

#ifdef DEBUG_SSL_CONNECTION
    SSL *ssl = m_ws.next_layer().native_handle();
    SSL_CTX* ssl_ctx = SSL_get_SSL_CTX(ssl);
    SSL_CTX_set_keylog_callback(ssl_ctx, ssl_keylog_callback);
#endif
    boost::asio::connect(beast::get_lowest_layer(m_ws), endpoint);
    
    m_ws.next_layer().handshake(ssl::stream_base::client);
    
    m_ws.set_option(websocket::stream_base::decorator(
        [](websocket::request_type& req) {
            req.set(boost::beast::http::field::user_agent, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36");
            req.set(boost::beast::http::field::origin, "http://localhost");
        }
    ));
    
    std::string path = std::format("/api/v2/channels/samsung.remote.control?name={}", APP_NAME_B64);
    if (m_token) {
        path += "&token=" + *m_token;
    }
    m_ws.handshake(TV_HOST + ":" + TV_PORT, path);
    read_event();
    heartbeat();
}

std::string key_to_string(Controller::Key key) {
    switch(key) {
        using enum Controller::Key;
        case POWER:
            return "KEY_POWER";
        case UP:
            return "KEY_UP";
        case DOWN:
            return "KEY_DOWN";
        case LEFT:
            return "KEY_LEFT";
        case RIGHT:
            return "KEY_RIGHT";
        case ENTER:
            return "KEY_ENTER";
        case BACK:
            return "KEY_RETURN";
        case PLAY:
            return "KEY_PLAY";
        case PAUSE:
            return "KEY_PAUSE";
        case MUTE:
            return "KEY_MUTE";
        case VOL_UP:
            return "KEY_VOLUP";
        case VOL_DOWN:
            return "KEY_VOLDOWN";
        case CH_UP:
            return "KEY_CHUP";
        case CH_DOWN:
            return "KEY_CHDOWN";
        case HOME:
            return "KEY_HOME";
        case MENU:
            return "KEY_MENU";
    }
    return "KEY_HOME";
}

void Controller::send_key(Controller::Key key) {
    json::object volume_up = {
        {"method", "ms.remote.control"},
        {"params", {
                {"Cmd", "Click"},
                {"DataOfCmd", key_to_string(key)},
                {"Option", "false"},
                {"TypeOfRemote", "SendRemoteKey"}
            }
        }
    };
    std::string payload = json::serialize(volume_up);
    m_ws.text(true);
    m_ws.write(boost::asio::buffer(payload));
}

void Controller::heartbeat() {
    m_ws.async_read(m_buffer, beast::bind_front_handler(&Controller::on_read, shared_from_this()));
}


void Controller::on_read(beast::error_code ec, std::size_t transfered) {
    if (ec) {
        return;
    }
    m_buffer.clear();
    heartbeat();
}

void Controller::close() {
    m_ws.close(beast::websocket::close_code::normal);
}
