#include "controller.hpp"
#include <boost/asio/io_context.hpp>
#include <boost/asio/connect.hpp>
#include <boost/asio/post.hpp>
#include <boost/beast/core.hpp>
#include <boost/beast/core/flat_buffer.hpp>
#include <boost/beast/http/status.hpp>
#include <boost/json.hpp>

constexpr std::string TV_HOST = "Samsung.local";
constexpr std::string TV_PORT = "8002";
constexpr std::string APP_NAME_B64 = "bXlTY3JpcHQ="; // "myScript"
constexpr std::string TOKEN = "15619641"; 


namespace beast = boost::beast;
namespace json = boost::json;

Controller::Controller(boost::asio::io_context &ioc): m_ioc(ioc),  m_ssl_ctx(ssl::context::tlsv12_client), m_ws(ioc, m_ssl_ctx) {
    tcp::resolver resolver{ioc};
    auto const endpoint = resolver.resolve(TV_HOST, TV_PORT, tcp::resolver::flags());
    
    boost::asio::connect(beast::get_lowest_layer(m_ws), endpoint);
    
    // 3. Perform the SSL/TLS Handshake
    m_ws.next_layer().handshake(ssl::stream_base::client);
    
    // 4. Configure specific WebSocket upgrade decoration decorators
    m_ws.set_option(websocket::stream_base::decorator(
        [](websocket::request_type& req) {
            req.set(boost::beast::http::field::user_agent, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36");
            req.set(boost::beast::http::field::origin, "http://localhost");
        }
    ));
    
    // 5. Build target API path handshake parameters
    std::string path = "/api/v2/channels/samsung.remote.control?name=" + APP_NAME_B64 + "&token=" + TOKEN;
    m_ws.handshake(TV_HOST + ":" + TV_PORT, path);
    boost::asio::post(m_ioc, [this]() {read_event();});
}

bool Controller::is_open() {
    return m_ws.is_open();
}

void Controller::read_event() {
    if (m_ws.is_open()) {
        beast::flat_buffer buffer;
        m_ws.read(buffer);
        buffer.clear();
        boost::asio::post(m_ioc, [this]() {read_event();});
    }
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
