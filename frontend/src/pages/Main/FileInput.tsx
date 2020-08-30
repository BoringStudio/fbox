import React from 'react';

import { useDropzone } from 'react-dropzone';
import { IconAdd } from './IconAdd';

export type IProps = {
  onDrop: (files: File[]) => void;
};

export const FileInput = (props: IProps) => {
  const onDrop = React.useCallback(
    acceptedFiles => {
      props.onDrop(acceptedFiles);
    },
    [props]
  );

  const { getRootProps, getInputProps, isDragActive } = useDropzone({ onDrop });

  return (
    <div {...getRootProps({ className: 'file-input' })}>
      <input {...getInputProps()} />
      <div className={`drop-zone${isDragActive ? ' active' : ''}`}>
        <IconAdd />
      </div>
    </div>
  );
};
