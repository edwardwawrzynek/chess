export const selectStyles = {
  option: (provided: any, state: any) => ({
    ...provided,
    color: 'white',
    backgroundColor: '#383c4a',
    padding: '0.2rem 0.4rem',
    fontSize: '0.8rem',
    fontWeight: state.isSelected ? 'bold' : 'normal',
  }),
  menu: (provided: any, state: any) => ({
    ...provided,
    backgroundColor: '#383c4a',
    border: '1px solid #bbb',
    borderRadius: '0',
    padding: '0',
  }),
  control: (provided: any, state: any) => ({
    ...provided,
    color: 'white',
    backgroundColor: '#383c4a',
    border: '1px solid #bbb',
    borderRadius: '0',
    padding: '0',
    minHeight: '0'
  }),
  singleValue: (provided: any, state: any) => ({
    ...provided,
    color: 'white',
    fontSize: '0.8rem',
    padding: '0',
    margin: '0',
  }),
  valueContainer: (provided: any, state: any) => ({
    ...provided,
    padding: '0',
    margin: '-0.8rem 0 -0.8rem 0.3rem',
  }),
  dropdownIndicator: (provided: any, state: any) => ({
    ...provided,
    maxHeight: '1.3rem',
    padding: '0'
  }),
  indicatorSeparator: (provided: any, state: any) => ({
    ...provided,
    margin: '2px 2px 2px 2px',
  })
};