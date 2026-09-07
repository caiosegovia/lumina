import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { api } from "./api";
import "./styles.css";

class AppErrorBoundary extends React.Component<React.PropsWithChildren, {failed:boolean}> {
  state={failed:false};
  static getDerivedStateFromError(){return{failed:true}}
  componentDidCatch(error:Error,info:React.ErrorInfo){void api.recordClientError("frontend_error",`${error.message} component=${info.componentStack?.slice(0,500)}`)}
  render(){return this.state.failed?<main className="fatal-recovery"><h1>O Lumina encontrou um erro de interface</h1><p>Seu catálogo e seus originais não foram alterados. Exporte o diagnóstico após reabrir o aplicativo.</p><button onClick={()=>location.reload()}>Reabrir interface</button></main>:this.props.children}
}

ReactDOM.createRoot(document.getElementById("root")!).render(<React.StrictMode><AppErrorBoundary><App /></AppErrorBoundary></React.StrictMode>);
