import cx from 'classnames';

export interface IconProps {
  name: string;
  size?: number;
  className?: string,
}

export const Icon: React.FC<IconProps> = function Icon({name, size, className}) {
  const style = size ? {
    width: `${size / 16}rem`,
    height: `${size / 16}rem`,
  } : {}
  
  return (
    <svg className={cx("icon", className)} style={style} aria-hidden="true">
      <use xlinkHref={`#icon-${name}`}></use>
    </svg>
  );
};
