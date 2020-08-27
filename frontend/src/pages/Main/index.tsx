import React from 'react';

import { StateContext } from '../../state';
import { SessionQrCode } from '../../components/SessionQrCode';
import { SessionCodeForm } from '../../components/SessionCodeForm';

import './style.scss';

export const MainPage = () => {
  const session = React.useContext(StateContext);

  if (session.state !== 'created') {
    return null;
  }

  return (
    <div className="content">
      <div className="panel panel--left">
        <SessionQrCode value={session.phrase} />
      </div>
      <div className="panel panel--right">
        <SessionCodeForm onSubmit={session.connect} />
      </div>
    </div>
  );
};
