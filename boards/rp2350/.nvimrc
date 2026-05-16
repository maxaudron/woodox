-- .nvimrc
vim.g.rustaceanvim = {
  server = {
    settings = {
      ["rust-analyzer"] = {
        cargo = { target = "thumbv8m.main-none-eabi" },
      }
    }
  }
}
