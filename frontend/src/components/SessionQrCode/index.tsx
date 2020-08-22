import React from 'react';
import QRCode from 'react-qr-code';

import './style.scss';

const FG_COLOR = '#1E1B19FF';
const BG_COLOR = '#00000000';

type SessionQrCodeProps = {
  value: string;
};

export const SessionQrCode = (props: SessionQrCodeProps) => {
  const { value } = props;

  return (
    <div className="session-qr-code">
      <div className="qr-code">
        <QRCode value={value} fgColor={FG_COLOR} bgColor={BG_COLOR} />
      </div>
      <div className="raw-code">{value}</div>
    </div>
  );
};
