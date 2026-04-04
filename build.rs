fn main() {
    // Этот скрипт запускается ДО компиляции основной программы
    let mut res = winres::WindowsResource::new();
    res.set_icon("icon.ico"); // Указываем имя нашего файла
    res.compile().unwrap(); // Вшиваем его в .exe
}
