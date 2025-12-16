pub fn get_req(path: &str) -> String {
    format! {
        "GET {} HTTP/1.1\r\n\
        Host: localhost\r\n\
        Connection: close\r\n\
        \r\n",
        path
    }
}
