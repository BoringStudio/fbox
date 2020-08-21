import React from 'react';

import { SessionCodeForm } from '../../components/SessionCodeForm';

import './style.scss';

export const MainPage = () => {
  return (
    <div className="content">
      <div className="panel panel--left">Left panel</div>
      <div className="panel panel--right">
        <SessionCodeForm />
      </div>
    </div>
  );
};
