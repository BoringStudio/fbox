import React from 'react';

import { StateContext, IStateCreated } from '../../state';
import { SessionQrCode } from '../../components/SessionQrCode';
import { SessionCodeForm } from '../../components/SessionCodeForm';

import './style.scss';

export const MainPage = () => {
  const session = React.useContext(StateContext) as IStateCreated;

  return (
    <div className="content main-page">
      <div className="panel panel--left">
        <SessionQrCode value={session.phrase} />
      </div>
      <div className="panel panel--right">
        <SessionCodeForm onSubmit={session.connect} />
      </div>
    </div>
  );
};
