import React, { useState } from "react";
import cx from "classnames";

import "./style.less";

type TabProps = React.PropsWithChildren<{ id: string; title: React.ReactNode }>;

export const Tab: React.FC<TabProps> = function Tab() {
  return null;
};

export const Tabs: React.FC<{
  defaultKey: string;
  children: React.ReactElement<TabProps, typeof Tab>[];
}> = function Tab({ defaultKey, children }) {
  const items = React.Children.map(children, (child) => {
    return child.props;
  });
  const ids = items.map((i) => i.id);
  if (!ids.length) return null;
  const [selected, setSelected] = useState(
    ids.find((id) => id === defaultKey) ?? ids[0]
  );
  return (
    <div className="mystar-tabs">
      <div className="mystar-tabs-header">
        {items.map((item) => (
          <div
            className={cx("mystar-tabs-header-item", {
              "mystar-tabs-header-item__active": item.id === selected,
            })}
            key={item.id}
            onClick={() => setSelected(item.id)}
          >
            {item.title}
          </div>
        ))}
      </div>
      <div className="mystar-tabs-body">
        {items
          .filter((item) => item.id === selected)
          .map((item) => item.children)}
      </div>
    </div>
  );
};
