import React from "react";

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  title?: string;
  subtitle?: string;
  actions?: React.ReactNode;
  variant?: "default" | "primary" | "secondary";
}

const Card = React.forwardRef<HTMLDivElement, CardProps>(
  (
    {
      children,
      title,
      subtitle,
      actions,
      variant = "default",
      className = "",
      ...props
    },
    ref,
  ) => {
    const baseClasses = "rounded-lg border bg-white shadow-sm";

    const variantClasses = {
      default: "border-gray-200",
      primary: "border-blue-200 bg-blue-50",
      secondary: "border-gray-200 bg-gray-50",
    };

    const classes = `${baseClasses} ${variantClasses[variant]} ${className}`;

    return (
      <div ref={ref} className={classes} {...props}>
        {(title || subtitle || actions) && (
          <div className="border-b border-gray-200 px-6 py-4">
            <div className="flex items-center justify-between">
              <div>
                {title && (
                  <h3 className="text-lg font-semibold text-gray-900">
                    {title}
                  </h3>
                )}
                {subtitle && (
                  <p className="mt-1 text-sm text-gray-500">{subtitle}</p>
                )}
              </div>
              {actions && <div>{actions}</div>}
            </div>
          </div>
        )}
        <div className="p-6">{children}</div>
      </div>
    );
  },
);

Card.displayName = "Card";

export default Card;
