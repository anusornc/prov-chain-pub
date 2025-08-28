import React from 'react';

interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  variant?: 'default' | 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info';
  size?: 'sm' | 'md' | 'lg';
}

const Badge = React.forwardRef<HTMLSpanElement, BadgeProps>(
  (
    {
      children,
      variant = 'default',
      size = 'md',
      className = '',
      ...props
    },
    ref
  ) => {
    const baseClasses = 'inline-flex items-center rounded-full font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2';
    
    const variantClasses = {
      default: 'bg-gray-100 text-gray-800 border border-gray-200',
      primary: 'bg-blue-100 text-blue-800 border border-blue-200',
      secondary: 'bg-gray-100 text-gray-800 border border-gray-200',
      success: 'bg-green-100 text-green-800 border border-green-200',
      warning: 'bg-yellow-100 text-yellow-800 border border-yellow-200',
      danger: 'bg-red-100 text-red-800 border border-red-200',
      info: 'bg-blue-100 text-blue-800 border border-blue-200'
    };
    
    const sizeClasses = {
      sm: 'text-xs px-2 py-0.5',
      md: 'text-sm px-2.5 py-0.5',
      lg: 'text-base px-3 py-1'
    };
    
    const classes = `${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]} ${className}`;
    
    return (
      <span ref={ref} className={classes} {...props}>
        {children}
      </span>
    );
  }
);

Badge.displayName = 'Badge';

export default Badge;
