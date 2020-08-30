import React from 'react';
import QRCode from 'react-qr-code';
import CopyToClipboard from 'react-copy-to-clipboard';

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
      <CopyToClipboard text={value}>
        <div className="raw-code">{value}</div>
      </CopyToClipboard>
    </div>
  );
};
