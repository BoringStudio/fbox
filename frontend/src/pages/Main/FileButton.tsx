import React from 'react';

import { FileInfo } from '../../state/sessionSocket';
import { IconDownload } from './IconDownload';

export type IProps = {
  seed: string;
  file: FileInfo;
};

const humanFileSize = (bytes: number, dp = 1) => {
  const thresh = 1024;

  if (Math.abs(bytes) < thresh) {
    return bytes + ' B';
  }

  const units = ['KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
  let u = -1;
  const r = 10 ** dp;

  do {
    bytes /= thresh;
    ++u;
  } while (Math.round(Math.abs(bytes) * r) / r >= thresh && u < units.length - 1);

  return bytes.toFixed(dp) + ' ' + units[u];
};

export const FileButton = (props: IProps) => {
  const { seed, file } = props;

  const downloadLink = new URL(
    `/v1/sessions/files/${file.id}?session_seed=${seed}`,
    process.env.REACT_APP_API_URL
  ).toString();

  return (
    <a className="file-button" href={downloadLink}>
      <IconDownload />
      <div className="label label--name">{file.name}</div>
      <div className="label label--size">{humanFileSize(file.size)}</div>
    </a>
  );
};
