//! Düzen-Baz çekirdeği. Tauri'ye bağlı olmayan tüm mantık burada durur; masaüstü uygulaması
//! (`duzenbaz`) ve komut satırı aracı (`duzenbaz-cli`) aynı kodu kullanır.
pub mod breaks;
pub mod clipboard;
pub mod db;
pub mod error;
pub mod history;
pub mod organizer;
pub mod renamer;
