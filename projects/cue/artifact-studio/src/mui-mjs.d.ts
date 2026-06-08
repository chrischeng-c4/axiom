declare module '@mui/icons-material/*.mjs' {
  const Icon: any
  export default Icon
}

declare module '@mui/material/*/index.mjs' {
  const Component: any
  export default Component
}

declare module '@mui/material/styles/ThemeProvider.mjs' {
  const ThemeProvider: any
  export default ThemeProvider
}

declare module '@mui/material/styles/createTheme.mjs' {
  const createTheme: any
  export default createTheme
}
