import React from 'react';

import { SessionQrCode } from '../../components/SessionQrCode';
import { SessionCodeForm } from '../../components/SessionCodeForm';

import './style.scss';

export const MainPage = () => {
  const [sessionCode, setSessionCode] = React.useState<string>(
    'shine high general turkey outer just'
  );

  return (
    <div className="content">
      <div className="panel panel--left">
        <SessionQrCode value={sessionCode} />
      </div>
      <div className="panel panel--right">
        <SessionCodeForm onSubmit={setSessionCode} />
      </div>
    </div>
  );
};
