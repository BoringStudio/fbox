import React from 'react';

import './style.scss';

export const LoadingPage = () => {
  return (
    <div className="content loading-page">
      <img src="/spinner.svg" alt="Loading..." />
    </div>
  );
};
