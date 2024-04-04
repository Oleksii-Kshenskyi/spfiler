# SPFiler - a Small Personal Filer for you and friends
A small personal filer to share files privately with your other PCs and your favorite people. Uses Rust/Tokio/Axum/etc.

# Goals

This is a small project intended for both educational and practical purposes, that is, for private file sharing between me and my friends, without relying on corporate file hosting services etc.

Now, as for the GOALS:
1. Written in Rust + Tokio + Axum
2. Needs to be able to share files P2P or with a metadata server between two or more people.
3. Needs to be secure (nothing crazy, the project should be within 2-4k lines of Rust code)
4. The web UI (JS? WASM?) would also be ideal, but it's a "nice to have" rather than a requirement.
5. Firstly, we need to just get the file sharing going through console tools, afterwards we'll maybe worry about GUI.

# Licensing

This project is under the usual MIT license. This means that the only thing you need is to remember to retain the copyright notice and the license text as long if you're using or modifying this code. Otherwise, feel free to have fun and do whatever you want with it! 