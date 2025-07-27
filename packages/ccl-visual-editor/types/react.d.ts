declare module 'react' {
  export interface MouseEvent<T = Element> {
    preventDefault(): void
    stopPropagation(): void
    currentTarget: EventTarget & T
    target: EventTarget | null
    clientX: number
    clientY: number
  }

  export interface DragEvent<T = Element> {
    preventDefault(): void
    dataTransfer: DataTransfer
    currentTarget: EventTarget & T
    clientX: number
    clientY: number
  }

  export interface RefObject<T> {
    readonly current: T | null
  }

  export function useState<S>(initialState: S | (() => S)): [S, (value: S | ((prev: S) => S)) => void]
  export function useCallback<T extends (...args: any[]) => any>(callback: T, deps: any[]): T
  export function useRef<T>(initialValue: T): RefObject<T>
  export function useRef<T = undefined>(): RefObject<T | undefined>
  export function useEffect(effect: () => void | (() => void), deps?: any[]): void

  export interface FC<P = {}> {
    (props: P): JSX.Element | null
  }

  export default React
  declare const React: {
    FC: typeof FC
    useState: typeof useState
    useCallback: typeof useCallback
    useRef: typeof useRef
    useEffect: typeof useEffect
  }
}

declare global {
  namespace JSX {
    interface IntrinsicElements {
      div: any
      svg: any
      path: any
      span: any
      h3: any
      p: any
      [elemName: string]: any
    }
    interface Element {}
    interface ElementClass {}
    interface ElementAttributesProperty {}
    interface ElementChildrenAttribute {}
    interface IntrinsicAttributes {}
    interface IntrinsicClassAttributes<T> {}
  }
} 