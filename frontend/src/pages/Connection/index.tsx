import React from 'react';

import { StateContext, IStateContext } from '../../state';
import { SessionQrCode } from '../../components/SessionQrCode';
import { SessionCodeForm } from '../../components/SessionCodeForm';

import './style.scss';

export const ConnectionPage = () => {
  const session = React.useContext(StateContext) as IStateContext<'created'>;

  return (
    <div className="content connection-page">
      <div className="panel panel--left">
        <SessionQrCode value={session.phrase} />
      </div>
      <div className="panel panel--right">
        <SessionCodeForm onSubmit={session.addPeer} />
      </div>
    </div>
  );
};
