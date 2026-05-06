## How to Run

1. Jalankan Server   
- Buka terminal di direktori proyek  
- Jalankan perintah `cargo run --bin server` untuk mengaktifkan server yang akan mendengarkan pada port 2000.

2. Buka Terminal Baru  
- Buka beberapa jendela terminal terpisah untuk menjalankan beberapa client secara bersamaan.

3. Jalankan Client  
- Di setiap terminal baru tersebut, jalankan perintah `cargo run --bin client`.

### Run One Server with Three Clients
![alt text](oneserverthreeclients.png)

When I type some text in one of the clients, that message is sent to the server and then broadcasted to all other connected clients. For example, when I type a message in Client 1, it appears in the terminal of Client 2 and Client 3, labeled with the sender's specific socket address. Based on the implementation, the server uses `tokio::select!` to concurrently handle receiving messages from a client and broadcasting them, while also listening for messages from other clients via the broadcast channel. I also noticed that the code prevents the message from being sent back to the original sender to avoid redundancy. This asynchronous flow ensures that all clients can communicate in real-time without blocking each other’s input or output.