import React from 'react';

interface TextAreaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  error?: string;
  helperText?: string;
  fullWidth?: boolean;
}

const TextArea = React.forwardRef<HTMLTextAreaElement, TextAreaProps>(
  (
    {
      label,
      error,
      helperText,
      fullWidth = false,
      className = '',
      id,
      ...props
    },
    ref
  ) => {
    const baseClasses = 'flex min-h-[80px] w-full rounded-md border bg-white px-3 py-2 text-sm transition-colors placeholder:text-gray-500 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50';
    
    const errorClasses = error 
      ? 'border-red-500 text-red-900 placeholder:text-red-500 focus-visible:ring-red-500' 
      : 'border-gray-300 focus-visible:ring-blue-500';
    
    const widthClass = fullWidth ? 'w-full' : '';
    
    const classes = `${baseClasses} ${errorClasses} ${widthClass} ${className}`;
    
    const textareaId = id || (label ? `textarea-${label.toLowerCase().replace(/\s+/g, '-')}` : undefined);
    
    return (
      <div className={fullWidth ? 'w-full' : ''}>
        {label && (
          <label 
            htmlFor={textareaId}
            className="block text-sm font-medium text-gray-700 mb-1"
          >
            {label}
          </label>
        )}
        <textarea
          ref={ref}
          id={textareaId}
          className={classes}
          {...props}
        />
        {(helperText || error) && (
          <p className={`mt-1 text-sm ${error ? 'text-red-600' : 'text-gray-500'}`}>
            {error || helperText}
          </p>
        )}
      </div>
    );
  }
);

TextArea.displayName = 'TextArea';

export default TextArea;
