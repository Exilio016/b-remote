#pragma once

#include <boost/asio/io_context.hpp>
#include <boost/asio/ssl/context.hpp>
#include <boost/beast/core/error.hpp>
#include <boost/beast/core/flat_buffer.hpp>
#include <boost/beast/websocket.hpp>
#include <boost/beast/websocket/ssl.hpp>
#include <boost/asio/ip/tcp.hpp>
#include <boost/asio/ssl/stream.hpp>
#include <cstddef>

namespace websocket = boost::beast::websocket;
using tcp = boost::asio::ip::tcp;
namespace ssl = boost::asio::ssl;

class Controller : public std::enable_shared_from_this<Controller> {
    boost::asio::io_context &m_ioc;
    ssl::context m_ssl_ctx;
    websocket::stream<ssl::stream<tcp::socket>> m_ws;
    std::optional<std::string> m_token;
    boost::beast::flat_buffer m_buffer;


public:
    enum class Key {
        POWER, 
        UP,
        DOWN,
        LEFT,
        RIGHT,
        ENTER,
        PLAY,
        PAUSE,
        BACK,
        MUTE, 
        VOL_UP, 
        VOL_DOWN, 
        CH_UP, 
        CH_DOWN,
        CH_LIST,
        HOME,
        MENU
    };
    
    Controller(boost::asio::io_context &ioc);
    void send_key(Key k);
    bool is_open();
    void connect();
    void close();
private:
    void read_event();
    void heartbeat();
    void on_read(boost::beast::error_code ec, std::size_t);
};
