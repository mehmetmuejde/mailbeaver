import type { ComponentProps, ReactNode } from "react";
import { Button as ButtonPrimitive } from "@/shared/ui/primitives/button";

type ButtonVariant = "default" | "outline" | "ghost" | "destructive";
type ButtonSize = "default" | "sm";

type ButtonProps = Omit<ComponentProps<typeof ButtonPrimitive>, "variant" | "size"> & {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** Every text button shows an icon before its label (design rule). */
  icon: ReactNode;
};

export function Button({
  variant = "outline",
  size = "default",
  type = "button",
  icon,
  children,
  ...props
}: ButtonProps) {
  return (
    <ButtonPrimitive type={type} variant={variant} size={size} {...props}>
      <span data-icon="inline-start" aria-hidden="true" className="inline-flex">
        {icon}
      </span>
      {children}
    </ButtonPrimitive>
  );
}
