import styled from 'styled-components'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import { RecoilRoot } from 'recoil'

import HomeView from './views/HomeView'

const router = createBrowserRouter([
  {
    path: '/',
    element: <HomeView />,
  },
  {
    path: '/:id',
    element: <HomeView />,
  }
])

const App = ({ className }) => {
  return (
    <div className={className}>
      <RecoilRoot>
        <RouterProvider router={router} />
      </RecoilRoot>
    </div>
  )
}

export default styled(App)`
  margin: 0;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
`
