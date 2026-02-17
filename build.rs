fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.set("CompanyName", "Thornvald");
        res.set("ProductName", "Stellar");
        res.set("FileDescription", "Stellar");
        res.set("LegalCopyright", "Copyright (C) Thornvald");
        res.set("InternalName", "stellar.exe");
        res.set("OriginalFilename", "stellar.exe");
        res.compile().expect("Failed to compile Windows resources");
    }
}
