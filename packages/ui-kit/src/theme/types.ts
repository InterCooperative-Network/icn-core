export interface ICNTokens {
  color: {
    primary: string
    secondary: string
    success: string
    warning: string
    danger: string
    background: string
    surface: string
    text: string
    textSecondary: string
    border: string
  }
  space: {
    xs: number
    sm: number
    md: number
    lg: number
    xl: number
  }
  size: {
    xs: number
    sm: number
    md: number
    lg: number
    xl: number
  }
  radius: {
    sm: number
    md: number
    lg: number
  }
  fontSize: {
    xs: number
    sm: number
    md: number
    lg: number
    xl: number
    xxl: number
  }
  fontWeight: {
    normal: number
    medium: number
    semibold: number
    bold: number
  }
}

export interface ICNTheme {
  name: string
  tokens: ICNTokens
  isDark: boolean
}

export type ThemeVariant = 'light' | 'dark' 