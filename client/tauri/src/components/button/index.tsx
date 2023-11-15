import cx from "classnames";
import "./style.less";

type ButtonProps = React.PropsWithChildren<{
  className?: string;
  onClick: () => void;
}>;

export const Button: React.FC<ButtonProps> = function (props) {
  return (
    <button
      onClick={props.onClick}
      className={cx("mystar-button", props.className)}
    >
      {props.children}
    </button>
  );
};
