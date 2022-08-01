import { Component, OnInit, TemplateRef } from '@angular/core';
import { ToastService } from '../../services/toast.service';

@Component({
  selector: 'app-toast',
  templateUrl: './toast.component.html',
  styleUrls: ['./toast.component.scss']
})
export class ToastComponent implements OnInit {

  constructor(private toastService: ToastService) { }

  ngOnInit(): void {
  }

  getToasts() {
    return this.toastService.toasts;
  }

  hideToast(toast: any) {
    this.toastService.remove(toast);
  }

  isTemplate(toast:any) {
    return toast.textOrTpl instanceof TemplateRef;
  }
}
