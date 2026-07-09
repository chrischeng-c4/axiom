import { Widget } from './Widget';
import type { Meta, StoryObj } from '@storybook/react';

const meta = {
  title: 'Components/Widget',
  component: Widget,
  decorators: [(Story) => <div className="jet-widget-frame"><Story /></div>],
  args: { label: 'Default', count: 0 },
} satisfies Meta<typeof Widget>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Interactive: Story = {
  argTypes: {
    count: { control: 'number' },
  },
  args: { label: 'Click me', count: 1 },
  play: async ({ canvasElement }) => {
    const button = canvasElement.querySelector('button');
    button?.click();
  },
};
