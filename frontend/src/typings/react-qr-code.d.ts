declare module 'react-qr-code' {
  import React from 'react';

  export interface QRCodeProps {
    value: string;
    size?: number;
    bgColor?: string;
    fgColor?: string;
    level?: 'L' | 'M' | 'Q' | 'H';
  }

  class QRCode extends React.Component<QRCodeProps, any> {}

  export default QRCode;
}
