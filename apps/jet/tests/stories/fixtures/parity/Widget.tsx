export interface WidgetProps {
  /** Label rendered inside the widget button. */
  label: string;
  /** Number of times the widget has been activated. */
  count: number;
}

/**
 * Widget component: a minimal interactive fixture used by the stories
 * parity fixture suite (decorators + argTypes + play + MDX together).
 */
export const Widget = (props: WidgetProps) => null;
